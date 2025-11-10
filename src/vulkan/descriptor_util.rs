#![allow(unsafe_op_in_unsafe_fn)]
use crate::game_objects::render_object::RenderObject;
use crate::vulkan::buffer_util::create_buffer;
use crate::vulkan::render_app::AppData;
use crate::vulkan::uniform_buffer_object::UniformBufferObject;
use anyhow::Result;
use vulkanalia::vk::{DeviceMemory, DeviceV1_0, HasBuilder};
use vulkanalia::{Device, Instance, vk};

pub unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    data.descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;
    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    uniform_buffers: &mut Vec<vk::Buffer>,
    uniform_buffers_memory: &mut Vec<DeviceMemory>,
) -> Result<()> {
    uniform_buffers.clear();
    uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (new_uniform_buffer, new_uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        uniform_buffers.push(new_uniform_buffer);
        uniform_buffers_memory.push(new_uniform_buffer_memory);
    }

    Ok(())
}

pub unsafe fn create_descriptor_pool(
    device: &Device,
    data: &mut AppData,
    max_objects: u32,
) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32 * max_objects);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32 * max_objects);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        // .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET) //?
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32 * max_objects);
    data.descriptor_pool = device.create_descriptor_pool(&info, None)?;
    Ok(())
}

pub unsafe fn create_descriptor_sets(
    device: &Device,
    data: &mut AppData,
    object: &mut RenderObject,
) -> Result<()> {
    // Allocate

    let layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    object.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update
    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(object.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(object.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(object.pbr.texture_data.image_view)
            .sampler(object.pbr.texture_data.sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(object.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(())
}

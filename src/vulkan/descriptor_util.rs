#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use crate::game_objects::render_object::{RenderObject, Renderable};
use crate::game_objects::scene::{Scene, Sun};
use crate::vulkan::buffer_util::create_buffer;
use crate::vulkan::uniform_buffer_object::{GlobalUniform, OrthographicLight, UniformBuffer};
use crate::vulkan::vertexbuffer_util::Vertex;
use crate::vulkan::winit_render_app::AppData;
use anyhow::Result;
use vulkanalia::vk::{DeviceMemory, DeviceV1_0, HasBuilder};
use vulkanalia::{Device, Instance, vk};
pub unsafe fn skybox_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    //camera and projection
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());
    //cubemap
    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    data.skybox_descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;
    Ok(())
}

pub unsafe fn gui_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let dims = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[dims, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    data.gui_descriptor_layout = device.create_descriptor_set_layout(&info, None)?;
    Ok(())
}

pub unsafe fn pbr_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    //camera and projection
    let camera = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());
    //orthographic lightsource
    let ortho_light = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());
    //object specific
    let object_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(2)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::all());
    //main color texture
    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(3)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[camera, ortho_light, object_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    data.pbr_descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

pub unsafe fn create_uniform_buffers<Ubo>(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    uniform_buffers: &mut Vec<vk::Buffer>,
    uniform_buffers_memory: &mut Vec<DeviceMemory>,
) -> Result<()>
where
    Ubo: UniformBuffer,
{
    uniform_buffers.clear();
    uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (new_uniform_buffer, new_uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<Ubo>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        uniform_buffers.push(new_uniform_buffer);
        uniform_buffers_memory.push(new_uniform_buffer_memory);
    }

    Ok(())
}

pub unsafe fn create_global_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
) -> Result<()> {
    data.global_buffer.clear();
    data.global_buffer_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (new_uniform_buffer, new_uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<GlobalUniform>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        data.global_buffer.push(new_uniform_buffer);
        data.global_buffer_memory.push(new_uniform_buffer_memory);
    }

    scene.sun.buffer.clear();
    scene.sun.memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (new_uniform_buffer, new_uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<OrthographicLight>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        scene.sun.buffer.push(new_uniform_buffer);
        scene.sun.memory.push(new_uniform_buffer_memory);
    }

    Ok(())
}

const GLOBAL_DESCRIPTOR_UNIFORMS: u32 = 60;
const GLOBAL_SAMPLERS: u32 = 3;
pub unsafe fn create_descriptor_pool(
    device: &Device,
    data: &mut AppData,
    max_objects: u32,
) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(
            data.swapchain_images.len() as u32 * (max_objects + GLOBAL_DESCRIPTOR_UNIFORMS),
        );

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32 * max_objects + GLOBAL_SAMPLERS);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET) //?
        .pool_sizes(pool_sizes)
        .max_sets(1024);
    data.descriptor_pool = device.create_descriptor_pool(&info, None)?;
    Ok(())
}

pub unsafe fn create_pbr_descriptor_sets<V, U>(
    device: &Device,
    data: &mut AppData,
    sun: &mut Sun,
    object: &mut RenderObject<V>,
) -> Result<()>
where
    V: Vertex,
    U: UniformBuffer,
{
    let layouts = vec![data.pbr_descriptor_set_layout; data.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    object.set_descriptor_sets(device.allocate_descriptor_sets(&info)?);

    // Update
    for i in 0..data.swapchain_images.len() {
        object.init_descriptor(device, data, sun, i);
    }

    Ok(())
}

pub unsafe fn create_skybox_descriptor_sets(
    device: &Device,
    data: &AppData,
    scene: &mut Scene,
) -> Result<()> {
    let layouts = vec![data.skybox_descriptor_set_layout; data.swapchain_images.len()];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);
    let Some(skybox) = &mut scene.skybox else {
        return Ok(());
    };
    skybox.descriptor_sets = device.allocate_descriptor_sets(&info)?;
    for i in 0..data.swapchain_images.len() {
        skybox.init_descriptor(device, data, i);
    }
    Ok(())
}

#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use crate::vulkan::winit_render_app::AppData;
use anyhow::{Result, anyhow};
use vulkanalia::vk::{DeviceV1_0, Handle, HasBuilder, InstanceV1_0};
use vulkanalia::{Device, Instance, vk};

/// used to create buffers for various functionss: vertex buffers, uniform buffers, etc.
pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(unsafe {
            get_memory_type_index(instance, data, properties, requirements)
        }?);

    let buffer_memory = unsafe { device.allocate_memory(&memory_info, None) }?;

    (unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0) })?;

    Ok((buffer, buffer_memory))
}

pub unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = unsafe { instance.get_physical_device_memory_properties(data.physical_device) };
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

pub unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer = unsafe { begin_single_time_commands(device, data) }?;

    let regions = vk::BufferCopy::builder().size(size);
    unsafe { device.cmd_copy_buffer(command_buffer, source, destination, &[regions]) };
    (unsafe { end_single_time_commands(device, data, command_buffer) })?;

    Ok(())
}

pub unsafe fn begin_single_time_commands(
    device: &Device,
    data: &AppData,
) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.single_time_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info) }?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    (unsafe { device.begin_command_buffer(command_buffer, &info) })?;

    Ok(command_buffer)
}

pub unsafe fn end_single_time_commands(
    device: &Device,
    data: &AppData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    (unsafe { device.end_command_buffer(command_buffer) })?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);
    unsafe { device.queue_submit(data.graphics_queue, &[info], vk::Fence::null()) }?;
    unsafe { device.queue_wait_idle(data.graphics_queue) }?;
    unsafe { device.free_command_buffers(data.single_time_pool, &[command_buffer]) };

    Ok(())
}

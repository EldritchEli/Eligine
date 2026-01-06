#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use crate::vulkan::MAX_FRAMES_IN_FLIGHT;
use crate::vulkan::queue_family_indices::QueueFamilyIndices;
use crate::vulkan::render_app::AppData;
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{Device, Instance, vk};

pub unsafe fn create_command_pools(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    for _ in 0..data.framebuffers.len() {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
            .queue_family_index(indices.graphics);
        data.command_centers.push(CommandCenter {
            command_pool: device.create_command_pool(&info, None)?,
            command_buffers: vec![],
        });
    }
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
        .queue_family_index(indices.graphics);
    data.single_time_pool = device.create_command_pool(&info, None)?;
    Ok(())
}

pub unsafe fn create_transient_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT) // Optional.
        .queue_family_index(indices.graphics);
    data.transient_command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}
#[derive(Clone, Debug, Default)]
pub struct CommandCenter {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

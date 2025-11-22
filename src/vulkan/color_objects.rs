use crate::vulkan::image_util::{create_image, create_image_view};
use crate::vulkan::render_app::AppData;
use vulkanalia::{Device, Instance, vk};

pub unsafe fn create_color_objects(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {
    let (color_image, color_image_memory) = unsafe {
        create_image(
            instance,
            device,
            data,
            data.swapchain_extent.width,
            data.swapchain_extent.height,
            1,
            1,
            data.msaa_samples,
            data.swapchain_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }?;

    data.color_image = color_image;
    data.color_image_memory = color_image_memory;

    data.color_image_view = unsafe {
        create_image_view(
            device,
            data.color_image,
            data.swapchain_format,
            vk::ImageAspectFlags::COLOR,
            1,
            1,
        )
    }?;

    Ok(())
}

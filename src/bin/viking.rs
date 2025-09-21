use VulcanEngine_0::vulkan::renderer;

use terrors::OneOf;
use winit::error::{EventLoopError, OsError};

use vulkanalia::vk::ErrorCode;
fn main() -> Result<(), OneOf<(OsError, anyhow::Error, EventLoopError, ErrorCode)>> {
    let vulkan_data = renderer::init("Eligine").map_err(OneOf::broaden)?;
    renderer::run(vulkan_data).map_err(OneOf::broaden)
}

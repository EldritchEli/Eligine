use glam::Mat4;
use vulkanalia::{Version, vk};
pub mod buffer_util;
pub mod color_objects;
pub mod command_buffer_util;
pub mod command_pool;
pub mod descriptor_util;
pub mod device_util;
pub mod framebuffer_util;
pub mod image_util;
pub mod input_state;
pub mod instance_util;
pub mod memory;
pub mod pipeline_util;
pub mod queue_family_indices;
pub mod render_pass_util;
pub mod shader_module_util;
pub mod shadow_map;
pub mod swapchain_util;
pub mod sync_util;
pub mod uniform_buffer_object;
pub mod vertexbuffer_util;
pub const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
pub const MAX_FRAMES_IN_FLIGHT: usize = 3;
pub const FAR_PLANE_DISTANCE: f32 = 100000.0;
pub const CORRECTION: Mat4 = Mat4::from_cols_array(&[
    1.0,
    0.0,
    0.0,
    0.0,
    // We're also flipping the Y-axis with this line's `-1.0`.
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0 / 2.0,
    0.0,
    0.0,
    0.0,
    1.0 / 2.0,
    1.0,
]);

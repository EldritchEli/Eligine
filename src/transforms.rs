
use nalgebra_glm::Mat4;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4
}



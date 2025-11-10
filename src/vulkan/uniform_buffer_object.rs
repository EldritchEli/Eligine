use glam::{Mat4, Vec4};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PushConstants {
    pub proj_inv_view: Mat4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct UniformBufferObject {
    pub view: Mat4,
    pub proj: Mat4,
    pub inv_view: Mat4,
    pub model: [Mat4; 10],
    pub base: Vec4,
}

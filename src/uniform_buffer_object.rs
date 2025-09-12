
use glam::Mat4;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
    pub inv_view: Mat4,

}



use std::ptr::copy_nonoverlapping as memcpy;

use glam::{Mat4, Vec4};
use vulkanalia::{
    Device,
    vk::{self, DeviceMemory, DeviceV1_0},
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PushConstants {
    pub proj_inv_view: Mat4,
}

pub trait UniformBuffer: Sized {
    unsafe fn map_memory(&self, device: &Device, mem: DeviceMemory) -> anyhow::Result<()> {
        let memory = unsafe {
            device.map_memory(
                mem,
                0,
                size_of::<PbrUniform>() as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        unsafe { memcpy(&self, memory.cast(), 1) };

        unsafe { device.unmap_memory(mem) };
        Ok(())
    }
}
impl UniformBuffer for PbrUniform {}
impl UniformBuffer for GlobalUniform {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PbrUniform {
    pub view: Mat4,
    pub proj: Mat4,
    pub inv_view: Mat4,
    pub model: [Mat4; 10],
    pub base: Vec4,
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct GlobalUniform {
    pub view: Mat4,
    pub proj: Mat4,
    //pub inv_view: Mat4,*/
    pub x: f32,
    pub y: f32,
    // pub elapsed: Float,
}

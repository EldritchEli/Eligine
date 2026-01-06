use std::mem::transmute;
use std::ptr::copy_nonoverlapping as memcpy;

use bevy::math::Vec3;
use glam::{Mat4, Vec4, vec4};
use vulkanalia::Instance;
use vulkanalia::{
    Device,
    vk::{self, DeviceMemory, DeviceV1_0},
};

use crate::vulkan::{descriptor_util::create_uniform_buffers, render_app::AppData};

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct PbrPushConstant {
    pub proj_inv_view: Mat4,
}
impl PbrPushConstant {
    pub fn data(&self) -> [u8; 64] {
        unsafe { transmute(self.proj_inv_view.to_cols_array()) }
    }
}

pub trait UniformBuffer: Sized {
    unsafe fn map_memory(&self, device: &Device, mem: DeviceMemory) -> anyhow::Result<()> {
        let memory = unsafe {
            device.map_memory(
                mem,
                0,
                size_of::<Self>() as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        unsafe { memcpy(&*self, memory.cast(), 1) };

        unsafe { device.unmap_memory(mem) };
        Ok(())
    }
}
impl UniformBuffer for PbrUniform {}
impl UniformBuffer for GlobalUniform {}
impl UniformBuffer for PointLight {}
impl UniformBuffer for OrthographicLight {}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PbrUniform {
    pub model: [Mat4; 10],
    pub base: Vec4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PointLight {
    pub model: Mat4,
    pub color: Vec4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct OrthographicLight {
    pub direction: Vec4,
    pub color: Vec4,
}
impl Default for OrthographicLight {
    fn default() -> Self {
        Self {
            direction: vec4(-3.0, 1.0, -1.0, 0.0).normalize(),
            color: Vec4::ONE,
        }
    }
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

impl GlobalUniform {
    pub unsafe fn init_buffer(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
    ) -> anyhow::Result<()> {
        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        unsafe {
            create_uniform_buffers::<GlobalUniform>(
                instance,
                device,
                data,
                &mut uniform_buffers,
                &mut uniform_buffers_memory,
            )?;
        };
        data.global_buffer = uniform_buffers;
        data.global_buffer_memory = uniform_buffers_memory;
        Ok(())
    }
}

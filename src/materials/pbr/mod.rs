use std::marker::PhantomData;

use bevy::ecs::component::Component;
use vulkanalia::{
    Device,
    vk::{self, WriteDescriptorSet},
};

use crate::{
    materials::{Material, MaterialInstance},
    vulkan::vertexbuffer_util::{Vertex, VertexPbr},
    winit_app::winit_render_app::AppData,
};
pub mod pipeline;
use pipeline::create_pbr_pipeline;

#[non_exhaustive]
pub struct PBR {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}
impl Material for PBR {
    type MaterialInstance = PbrRenderSource<VertexPbr>;

    fn pipeline_layout(&self) -> vulkanalia::vk::PipelineLayout {
        self.pipeline_layout
    }

    fn pipeline(&self) -> vulkanalia::vk::Pipeline {
        self.pipeline
    }

    fn reload_pipeline(
        &mut self,
        device: &mut Device,
        data: &mut AppData,
        subpass_order: u32,
    ) -> anyhow::Result<()> {
        let (pipeline, pipeline_layout) =
            unsafe { create_pbr_pipeline(device, data, subpass_order)? };
        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;
        Ok(())
    }

    fn descriptor_set_layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        todo!()
    }

    fn descriptor_set(&self, instance: &Self::MaterialInstance) -> vulkanalia::vk::DescriptorSet {
        todo!()
    }

    fn draw(
        &self,
        device: &mut vulkanalia::Device,
        commands: &vulkanalia::vk::CommandBuffer,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn new(&self, device: &mut vulkanalia::Device, data: &AppData, subpass_order: u32) -> Self {
        todo!()
    }
}
//Represents the an instance of a pbr material. with support for instanced rendering.
#[derive(Component)]
pub struct PbrRenderSource<V>
where
    V: Vertex,
{
    indices: Vec<u32>,
    vertices: Vec<V>,
    pbr: PBR,
    uniform_buffer: vk::Buffer,
    uniform_memory: vk::DeviceMemory,
    uniform_mem_map: usize,
    descriptor_set: usize,
    buffer_size: u32,
    instance_count: usize,
    empty_space_offsets: Vec<usize>,
}

impl<V: Vertex> MaterialInstance for PbrRenderSource<V> {}

#[derive(Component)]
pub struct UniformInstance<M: MaterialInstance + Sync> {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mem_map: usize,
    buffer_offset: u32,
    material: PhantomData<M>,
    changed: bool,
    //write_descriptor: Vec<WriteDescriptorSet>,
}

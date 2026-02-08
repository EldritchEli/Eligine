use vulkanalia::vk::{CommandBuffer, DescriptorSet, DescriptorSetLayout, Pipeline, PipelineLayout};

use crate::winit_app::winit_render_app::AppData;

pub trait Material {
    type MaterialInstance;
    fn new(&self, device: &mut vulkanalia::Device, data: &AppData, subpass_order: u32) -> Self;
    fn pipeline_layout(&self) -> PipelineLayout;
    fn pipeline(&self) -> Pipeline;
    fn reload_pipeline(
        &mut self,
        device: &mut vulkanalia::Device,
        data: &mut AppData,
        subpass_order: u32,
    ) -> anyhow::Result<()>;
    fn descriptor_set_layout(&self) -> DescriptorSetLayout;
    fn descriptor_set(&self, instance: &Self::MaterialInstance) -> DescriptorSet;
    fn draw(&self, device: &mut vulkanalia::Device, commands: &CommandBuffer)
    -> anyhow::Result<()>;
}
pub trait MaterialInstance: Sync + std::marker::Sync + Send + std::marker::Send {}

pub mod pbr;

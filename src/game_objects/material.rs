use vulkanalia::vk::{CommandBuffer, DescriptorSet, DescriptorSetLayout, Pipeline, PipelineLayout};

pub trait Material {
    type MaterialInstance;
    fn pipeline_layout(&self) -> PipelineLayout;
    fn pipeline(&self) -> Pipeline;
    fn reload_pipeline(&mut self) -> anyhow::Result<()>;
    fn descriptor_set_layout(&self) -> DescriptorSetLayout;
    fn descriptor_set(&self, instance: &Self::MaterialInstance) -> DescriptorSet;
    fn draw(&self, device: &mut vulkanalia::Device, commands: &CommandBuffer)
    -> anyhow::Result<()>;
}
pub trait MaterialInstance {}

use vulkanalia::vk::{DescriptorSet, DescriptorSetLayout, Pipeline, PipelineLayout};

pub trait Material {
    type MaterialInstance;
    fn pipeline_layout(&self) -> PipelineLayout;
    fn pipeline(&self) -> Pipeline;
    fn descriptor_set_layout(&self) -> DescriptorSetLayout;
    fn descriptor_set(&self, instance: &Self::MaterialInstance) -> DescriptorSet;
}
pub trait MaterialInstance<M: Material> {}

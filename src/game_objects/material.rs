use vulkanalia::vk::{DescriptorSet, PipelineLayout};

#[derive(Debug, Clone)]
pub struct Material {
    descriptor_set: Vec<DescriptorSet>,
    pipeline_layout: PipelineLayout,
}

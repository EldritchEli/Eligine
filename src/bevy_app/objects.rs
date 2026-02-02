use bevy::ecs::bundle::{self, Bundle};
use bevy::ecs::component::Component;
use bevy::transform::components::Transform;
use vulkanalia::vk;

use crate::vulkan::vertexbuffer_util::Vertex;

#[derive(Component)]
pub struct Name(String);

#[derive(Bundle)]
pub struct PbrInstance {
    name: Name,
    transform: Transform,
    uniform: UniformInstance,
}

#[derive(Component)]
pub struct PbrRenderObject<V>
where
    V: Vertex,
{
    indices: Vec<u32>,
    vertices: Vec<V>,
    uniform_buffer: vk::Buffer,
    uniform_memory: vk::DeviceMemory,
    uniform_mem_map : usize,
    instance_count : usize,
}

#[derive(Component)]
pub struct UniformInstance {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mem_map: usize,
    buffer_offset: u32,
}





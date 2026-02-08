use std::marker::PhantomData;

use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::transform::components::Transform;
use vulkanalia::vk;

use crate::game_objects::material::MaterialInstance;
use crate::game_objects::render_object::PBR;
use crate::vulkan::vertexbuffer_util::Vertex;

#[derive(Component)]
pub struct Name(String);

#[derive(Bundle)]
pub struct PbrInstance<M: MaterialInstance + 'static> {
    name: Name,
    transform: Transform,
    uniform: UniformInstance<M>,
}

//Represents the an instance of a pbr material. with support for instances rendering.
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
}

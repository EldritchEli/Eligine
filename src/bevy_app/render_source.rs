use std::marker::PhantomData;

use bevy::ecs::component::Component;
use vulkanalia::vk::{self, WriteDescriptorSet};

use crate::{
    game_objects::{material::MaterialInstance, render_object::PBR},
    vulkan::vertexbuffer_util::Vertex,
};

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
    write_descriptor: Vec<WriteDescriptorSet>,
}

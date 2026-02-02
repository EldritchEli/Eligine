use bevy::math::Vec4;
use serde::{Deserialize, Serialize};

use crate::{
    game_objects::material::{Material, MaterialInstance},
    vulkan::vertexbuffer_util::Vertex,
};
pub struct UnstagedPbrData {
    base_image: Option<Vec<u8>>,
    base: Vec4,
}
pub struct UnstagedRenderObject<V: Vertex, M: MaterialInstance> {
    vertices: Vec<V>,
    indices: Vec<u32>,
    material: M,
}

//scene should be used to save and load scenes,
#[derive(Serialize, Deserialize)]
pub struct Scene {}

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use nalgebra_glm::{vec2, vec3};
use terrors::OneOf;
use uuid::Uuid;
use tobj::{LoadError, Mesh};
use vulkanalia::vk::{Buffer, DescriptorSet, DeviceMemory};
use crate::game_objects::transform::Transform;
use crate::render_app::AppData;
use crate::vertexbuffer_util::{ Texture, Vertex};

#[derive(Clone,Debug)]
pub struct RenderObject {
  vertices: Vec<Vertex>,
  indices: Vec<u32>,

  texture: Texture,
  instances: HashMap<Uuid, RenderInstance>
}


impl RenderObject {



  pub fn load_model(path: PathBuf) -> Result<(Vec<Vertex>,Vec<u32>), OneOf<(io::Error,LoadError)>> {
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut reader = BufReader::new(File::open(path)
      .map_err(OneOf::broaden)?);

    let (models,materials) = tobj::load_obj_buf(
      &mut reader,
      &tobj::LoadOptions { triangulate: true, ..Default::default() },
      |_| Ok(Default::default()),
    ).map_err(OneOf::broaden)?;

    let mut unique_vertices = HashMap::new();

    for model in &models {
      for index in &model.mesh.indices {
        let pos_offset = (3 * index) as usize;
        let tex_coord_offset = (2 * index) as usize;
        let vertex = Vertex {
          pos: vec3(
            model.mesh.positions[pos_offset],
            model.mesh.positions[pos_offset + 1],
            model.mesh.positions[pos_offset + 2],
          ),
          color: vec3(1.0, 1.0, 1.0),
          tex_coord: vec2(
            model.mesh.texcoords[tex_coord_offset],
            1.0 - model.mesh.texcoords[tex_coord_offset + 1],
          ),

        };


        if let Some(index) = unique_vertices.get(&vertex) {
          indices.push(*index as u32);

        } else {
          let index = vertices.len();
          unique_vertices.insert(vertex, index);
          vertices.push(vertex);
          indices.push(index as u32);
        }
      }
    }
    Ok((vertices, indices))
  }

  pub fn new(model_path: PathBuf, texture: PathBuf) -> Result<Self, OneOf<(io::Error,LoadError)>> {

    let (vertices, indices) = RenderObject::load_model(model_path)?;
    Self {
      vertices,
      indices,
      texture,
      instances: HashMap::new(),
    }
  }
  pub fn insert_instance(&mut self, instance: RenderInstance) -> Result<Uuid,RenderInstance> {
    let id = Uuid::new_v4();
    match self.instances.insert(id, instance) {

      Some(instance) => Err(instance),
      None => Ok(id)
    }
  }

  pub fn remove_instance(&mut self, id: &Uuid) -> Option<RenderInstance> {
    self.instances.remove(id)
  }
}



#[derive(Clone,Debug)]
pub struct RenderInstance {
 pub transform: Transform,
 pub uniform_buffer: Buffer,
 pub uniform_buffer_memory: DeviceMemory,
 pub uniform_buffers_mapped: Uuid,
 pub descriptor_sets: Vec<DescriptorSet>,

}
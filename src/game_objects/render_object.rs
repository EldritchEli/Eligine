use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use nalgebra_glm::{vec2, vec3};
use terrors::OneOf;
use uuid::Uuid;
use tobj::{LoadError, Mesh};
use vulkanalia::{Device, Instance};
use vulkanalia::loader::LoaderError;
use vulkanalia::vk::{Buffer, DescriptorSet, DeviceMemory};
use crate::descriptor_util::{create_descriptor_sets, create_uniform_buffers};
use crate::game_objects::transform::Transform;
use crate::image_util::TextureData;
use crate::render_app::AppData;
use crate::vertexbuffer_util::{Texture, Vertex, VertexData};

#[derive(Clone,Debug)]
pub struct RenderObject {
    pub vertex_data: VertexData,
    pub texture_data: TextureData,
    pub uniform_buffers: Vec<Buffer>,
    pub uniform_buffers_memory: Vec<DeviceMemory>,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub instances: HashMap<Uuid, RenderInstance>
}


impl RenderObject {
    pub unsafe fn load(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        model_path: PathBuf,
        image_path: PathBuf)
        -> Result<RenderObject, OneOf<(io::Error, LoadError, anyhow::Error)>> {
        let (vertices, indices) = Self::load_model(model_path)
          .map_err(OneOf::broaden)?;
        let texture_data = TextureData::create_texture(instance, device, data, image_path)
          .map_err(|e| OneOf::new(e))?;
        let vertex_data = VertexData::create_vertex_data(instance, device, data, vertices, indices)
          .map_err(|e| OneOf::new(e))?;
        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        create_uniform_buffers(instance, device, data, &mut uniform_buffers, &mut uniform_buffers_memory)
          .map_err(|e| OneOf::new(e))?;
        let mut object = Self {
          vertex_data,
          texture_data,
          uniform_buffers,
          uniform_buffers_memory,
          descriptor_sets: vec![],
          instances: Default::default(), };
        create_descriptor_sets(device, data, &mut object).map_err(|e| OneOf::new(e))?;
        Ok(object)
  }



  pub fn load_model(model_path: PathBuf) -> Result<(Vec<Vertex>,Vec<u32>), OneOf<(io::Error,LoadError)>> {
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut reader = BufReader::new(File::open(model_path)
      .map_err(|e | OneOf::new(e))?);

    let (models,materials) = tobj::load_obj_buf(
      &mut reader,
      &tobj::LoadOptions { triangulate: true, ..Default::default() },
      |_| Ok(Default::default()),
    ).map_err(|e| OneOf::new(e))?;

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

  pub fn insert_instance(&mut self, instance: RenderInstance) -> Result<Uuid,RenderInstance> {
    let id = Uuid::new_v4();
    match self.instances.insert(id, instance) {

      Some(instance) => Err(instance),
      None => Ok(id)
    }
  }

  pub fn insert_from_transform(&mut self,transform: Transform ) -> Result<Uuid,RenderInstance> {
    self.insert_instance(RenderInstance {transform})
  }
  pub fn remove_instance(&mut self, id: &Uuid) -> Option<RenderInstance> {
    self.instances.remove(id)
  }
}



#[derive(Clone,Debug)]
pub struct RenderInstance {
 pub transform: Transform,
 //pub uniform_buffer: Buffer,
 //pub uniform_buffer_memory: DeviceMemory,
 //pub uniform_buffers_mapped: Uuid,
 //pub descriptor_sets: Vec<DescriptorSet>,

}
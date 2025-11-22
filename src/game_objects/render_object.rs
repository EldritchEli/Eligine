use crate::game_objects::transform::Transform;
use crate::vulkan::descriptor_util::{create_pbr_descriptor_sets, create_uniform_buffers};
use crate::vulkan::image_util::TextureData;
use crate::vulkan::render_app::AppData;
use crate::vulkan::uniform_buffer_object::{PbrUniform, UniformBuffer};
use crate::vulkan::vertexbuffer_util::{Vertex, VertexData, VertexPbr};
use glam::{Vec4, vec2, vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use terrors::OneOf;
use tobj::LoadError;

use vulkanalia::vk::{
    self, Buffer, DescriptorImageInfo, DescriptorSet, DeviceMemory, DeviceV1_0, HasBuilder,
    WriteDescriptorSet,
};
use vulkanalia::{Device, Instance};

#[derive(Copy, Clone, Debug)]
pub struct RenderId(pub usize);
#[derive(Clone, Copy, Debug)]
pub struct ObjectId(pub usize);

#[derive(Clone, Debug)]
pub struct PBR {
    pub texture_data: TextureData,
    pub base: Vec4,
}

pub trait Renderable {
    fn set_descriptor_sets(&mut self, descriptor_sets: Vec<DescriptorSet>);
    fn get_descriptor_sets(&self) -> &Vec<DescriptorSet>;
    fn get_uniform_buffers(&self) -> &Vec<Buffer>;
    fn init_descriptor(&self, device: &Device, i: usize);
}
impl<V> Renderable for RenderObject<V>
where
    V: Vertex,
{
    fn set_descriptor_sets(&mut self, descriptor_sets: Vec<DescriptorSet>) {
        self.descriptor_sets = descriptor_sets;
    }

    fn get_descriptor_sets(&self) -> &Vec<DescriptorSet> {
        &self.descriptor_sets
    }

    fn get_uniform_buffers(&self) -> &Vec<Buffer> {
        &self.uniform_buffers
    }

    fn init_descriptor(&self, device: &Device, i: usize) {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(self.get_uniform_buffers()[i])
            .offset(0)
            .range(size_of::<PbrUniform>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.pbr.texture_data.image_view)
            .sampler(self.pbr.texture_data.sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        unsafe {
            device.update_descriptor_sets(
                &[ubo_write, sampler_write],
                &[] as &[vk::CopyDescriptorSet],
            )
        };
    }
}
#[derive(Debug, Clone)]
pub struct RenderObject<V>
where
    V: Vertex,
{
    pub vertex_data: VertexData<V>,
    pub pbr: PBR,
    pub uniform_buffers: Vec<Buffer>,
    pub uniform_buffers_memory: Vec<DeviceMemory>,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub instances: Vec<ObjectId>,
}

impl<V> RenderObject<V>
where
    V: Vertex,
{
    pub unsafe fn create_render_object(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        vertex_data: VertexData<V>,
        pbr: PBR,
    ) -> Result<RenderObject<V>, OneOf<(String, anyhow::Error)>> {
        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        (unsafe {
            create_uniform_buffers::<PbrUniform>(
                instance,
                device,
                data,
                &mut uniform_buffers,
                &mut uniform_buffers_memory,
            )
            .map_err(OneOf::new)
        })?;
        let mut object = Self {
            vertex_data,
            pbr,
            uniform_buffers,
            uniform_buffers_memory,
            descriptor_sets: vec![],
            instances: Default::default(),
        };
        (unsafe {
            create_pbr_descriptor_sets::<V, PbrUniform>(device, data, &mut object)
                .map_err(|e| OneOf::new(e))
        })?;
        Ok(object)
    }
}

/* pub unsafe fn load_obj_format(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        model_path: PathBuf,
        image_path: PathBuf,
    ) -> Result<RenderObject<V>, OneOf<(io::Error, LoadError, String, anyhow::Error)>> {
        let (vertices, indices) = Self::load_model(model_path).map_err(OneOf::broaden)?;
        let texture_data = unsafe {
            TextureData::create_texture_from_path(instance, device, data, image_path)
                .map_err(|e| OneOf::new(e))
        }?;
        let vertex_data = unsafe {
            VertexData::create_vertex_data(instance, device, data, vertices, indices)
                .map_err(|e| OneOf::new(e))
        }?;
        unsafe {
            Self::create_render_object(
                instance,
                device,
                data,
                vertex_data,
                PBR {
                    texture_data,
                    base: Vec4::ONE,
                },
            )
            .map_err(OneOf::broaden)
        }
    }
*/
/*pub fn load_model(
    model_path: PathBuf,
) -> Result<(Vec<V>, Vec<u32>), OneOf<(io::Error, LoadError)>> {
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut reader = BufReader::new(File::open(model_path).map_err(|e| OneOf::new(e))?);

    let (models, _materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )
    .map_err(|e| OneOf::new(e))?;

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
}*/

#![allow(unsafe_op_in_unsafe_fn)]
use anyhow::Result;
use std::mem::size_of;

use crate::vulkan::buffer_util::{copy_buffer, create_buffer};
use crate::vulkan::render_app::AppData;
use glam::{Vec2, Vec3, vec3};
use std::hash::{Hash, Hasher};
use std::ptr::copy_nonoverlapping as memcpy;
use varlen_macro::define_varlen;
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{Device, Instance, vk};

#[repr(C)]
#[derive(Debug, Clone)]
/// texture coordinates and paths to texture file
pub struct Texture {
    pub tex_string: String,
    pub tex_coords: Vec<Vec2>,
}
/// color is either encoded as RGB triplets or texture coordinates and paths to texture file
#[repr(C)]
#[derive(Debug)]
pub enum Colors {
    RGB(Vec<Vec3>),
    Texture(Texture),
}

pub enum Attribute {
    VertexPbr,
    Normal,
    TexCoord,
}
pub unsafe fn quad_vertex_data(
    width: u32,
    height: u32,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<VertexData<SimpleVertex>> {
    let vertices = vec![
        SimpleVertex { pos: Vec3::ZERO },
        SimpleVertex {
            pos: vec3(0.0, height as f32, 0.0),
        },
        SimpleVertex {
            pos: vec3(width as f32, 0.0, 0.0),
        },
        SimpleVertex {
            pos: vec3(width as f32, height as f32, 0.0),
        },
    ];
    let indices = vec![0, 2, 1, 1, 2, 3];
    unsafe { VertexData::create_vertex_data(instance, device, data, vertices, indices) }
}
#[derive(Clone, Debug, Default)]
pub struct VertexData<V>
where
    V: Vertex,
{
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
}

impl<V> VertexData<V>
where
    V: Vertex,
{
    pub unsafe fn create_vertex_data(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Result<Self> {
        let (vertex_buffer, vertex_buffer_memory) =
            unsafe { Self::create_vertex_buffer(instance, device, data, &vertices) }?;
        let (index_buffer, index_buffer_memory) =
            unsafe { Self::create_index_buffer(instance, device, data, &indices) }?;
        Ok(VertexData {
            vertices,
            indices,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        })
    }
    pub unsafe fn create_vertex_buffer(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        vertices: &Vec<V>,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let size = (size_of::<VertexPbr>() * vertices.len()/*VERTICES.len()*/) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory =
            device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

        memcpy(
            vertices.as_ptr(), /*VERTICES.as_ptr()*/
            memory.cast(),
            vertices.len(),
        );

        device.unmap_memory(staging_buffer_memory);

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((vertex_buffer, vertex_buffer_memory))
    }

    pub unsafe fn create_index_buffer(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        indices: &Vec<u32>,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let size = (size_of::<u32>() * indices.len()/*INDICES.len()*/) as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory =
            device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

        memcpy(indices.as_ptr(), memory.cast(), indices.len());

        device.unmap_memory(staging_buffer_memory);

        let (index_buffer, index_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(device, data, staging_buffer, index_buffer, size)?;

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((index_buffer, index_buffer_memory))
    }
}
/*pub fn load_model(data: &mut AppData, path: PathBuf) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);

    let (models,materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { triangulate: true, ..Default::default() },
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;
            let VertexPbr = VertexPbr {
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


            if let Some(index) = unique_vertices.get(&VertexPbr) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                unique_vertices.insert(VertexPbr, index);
                data.vertices.push(VertexPbr);
                data.indices.push(index as u32);
            }


        }
    }
    Ok(())
}*/

#[repr(C)]
#[define_varlen]
pub struct MeshData {
    #[controls_layout]
    pub s: usize,
    #[varlen_array]
    pub positions: [Vec3; *s],
    #[varlen_array]
    pub normals: [Vec3; *s],
    pub indices: Option<Vec<u16>>,
    pub colors: Colors,
}
/*

impl MeshData {
    pub fn new(vertices: Vec<Vec3>, indices: &[u16], colors: &[Colors]) -> Result<MeshData> {

        let v = vec![9];
    }

    }

*/

/*
pub fn test_mesh1() -> MeshData{ MeshData {
    positions: [vec3(-0.5, -0.5, 0.0),
        vec3(0.5, -0.5, 0.0),
        vec3(0.5, 0.5, 0.0),
        vec3(-0.5, 0.5, 0.0),
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, -0.5)],
    normals: None,
    indices: Some(Vec::from([0u16, 1u16, 2u16, 2u16, 3u16, 0u16,
        4u16, 5u16, 6u16, 6u16, 7u16, 4u16])),
    VertexPbr_count: 8,
    colors: Colors::Texture(Texture {
        tex_string: "src/resources/birk.png".to_string(),
        tex_coords: Vec::from([vec2(1.0, 0.0), vec2(0.0, 0.0),
            vec2(0.0, 1.0), vec2(1.0, 1.0),
            vec2(1.0, 0.0), vec2(0.0, 0.0),
            vec2(0.0, 1.0), vec2(1.0, 1.0, )]
        ),
    })
}
}*/
pub trait Vertex {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<VertexPbr>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexPbr {
    pub pos: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
}

impl VertexPbr {
    pub const fn new(pos: Vec3, normal: Vec3, tex_coord: Vec2) -> Self {
        Self {
            pos,
            normal,
            tex_coord,
        }
    }
}
impl Vertex for VertexPbr {
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let normal = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32)
            .build();
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32)
            .build();
        vec![pos, normal, tex_coord]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SimpleVertex {
    pos: Vec3,
}

impl Vertex for SimpleVertex {
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0)
                .build(),
        ]
    }
}

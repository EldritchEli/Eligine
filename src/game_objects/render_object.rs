use crate::game_objects::scene::Sun;
use crate::vulkan::descriptor_util::{create_pbr_descriptor_sets, create_uniform_buffers};
use crate::vulkan::image_util::TextureData;
use crate::vulkan::render_app::AppData;
use crate::vulkan::uniform_buffer_object::{GlobalUniform, OrthographicLight, PbrUniform};
use crate::vulkan::vertexbuffer_util::{Vertex, VertexData};
use glam::Vec4;
use terrors::OneOf;

use vulkanalia::vk::{self, Buffer, DescriptorSet, DeviceMemory, DeviceV1_0, HasBuilder};
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
    fn init_descriptor(&self, device: &Device, data: &AppData, sun: &mut Sun, i: usize);
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

    fn init_descriptor(&self, device: &Device, data: &AppData, sun: &mut Sun, i: usize) {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(sun.buffer[i])
            .offset(0)
            .range(size_of::<OrthographicLight>() as u64);
        let buffer_info = &[info];
        let ortho_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.global_buffer[i])
            .offset(0)
            .range(size_of::<GlobalUniform>() as u64);

        let buffer_info = &[info];
        let global_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(self.get_uniform_buffers()[i])
            .offset(0)
            .range(size_of::<PbrUniform>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(2)
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
            .dst_binding(3)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        unsafe {
            device.update_descriptor_sets(
                &[ortho_write, global_write, ubo_write, sampler_write],
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
        sun: &mut Sun,
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
            create_pbr_descriptor_sets::<V, PbrUniform>(device, data, sun, &mut object)
                .map_err(|e| OneOf::new(e))
        })?;
        Ok(object)
    }
}

use std::{fs::File, io::BufReader};

use vulkanalia::{
    Device, Instance,
    vk::{self, DescriptorSet, DeviceV1_0, HasBuilder},
};

use crate::vulkan::{
    descriptor_util::create_uniform_buffers,
    image_util::TextureData,
    render_app::AppData,
    uniform_buffer_object::{self, GlobalUniform},
};

impl SkyBox {
    pub fn get_descriptor_sets(&self) -> &Vec<vulkanalia::vk::DescriptorSet> {
        &self.descriptors
    }

    pub fn init_descriptor(&self, device: &Device, data: &AppData, i: usize) {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.global_buffer[i])
            .offset(0)
            .range(size_of::<GlobalUniform>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.get_descriptor_sets()[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.texture_data.image_view)
            .sampler(self.texture_data.sampler);

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

#[derive(Debug)]
pub struct SkyBox {
    pub texture_data: TextureData,
    pub descriptors: Vec<DescriptorSet>,
}
impl SkyBox {
    pub fn load(
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        up: &str,
        down: &str,
        left: &str,
        right: &str,
        back: &str,
        front: &str,
    ) -> anyhow::Result<Self> {
        let mut pixels = vec![];
        let mut x = 0;
        let mut y = 0;
        for file in [right, left, up, down, front, back] {
            let decoder = png::Decoder::new(BufReader::new(File::open(file).unwrap()));
            let mut reader = decoder.read_info().unwrap();
            // Allocate the output buffer.
            let mut buf = vec![0; reader.output_buffer_size()];
            // Read the next frame. An APNG might contain multiple frames.
            let info = reader.next_frame(&mut buf).unwrap();
            println!("buffer size: {:?}", info.buffer_size());
            if x == 0 && y == 0 {
                x = info.width;
                y = info.height
            } else {
                assert_eq!(x, info.width);
                assert_eq!(y, info.height);
            }
            // Grab the bytes of the image.
            let mut bytes: Vec<u8> = buf.into_iter().take(info.buffer_size()).collect();
            pixels.append(&mut bytes);
        }
        println!("pixel sizes: {:?}", pixels.len());
        println!("x: {:?}, y: {:?}", x, y);

        let texture_data = unsafe {
            TextureData::create_cubemap_from_data(instance, device, data, pixels, (x, y))
        }?;

        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        unsafe {
            create_uniform_buffers::<GlobalUniform>(
                instance,
                device,
                data,
                &mut uniform_buffers,
                &mut uniform_buffers_memory,
            )?;
        };
        data.global_buffer = uniform_buffers;
        data.global_buffer_memory = uniform_buffers_memory;
        return Ok(Self {
            texture_data,
            descriptors: vec![],
        });
    }
}

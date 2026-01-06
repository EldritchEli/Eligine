#![allow(unsafe_op_in_unsafe_fn)]
use std::collections::HashMap;

use egui::{
    ClippedPrimitive, Color32, Context, FullOutput, Rect, RichText, SidePanel, TextureId, Ui,
};
use glam::{U8Vec4, Vec2};
use log::info;
use vulkanalia::{
    Device, Instance,
    vk::{self, DeviceV1_0, HasBuilder},
};
use winit::window;

use crate::vulkan::{
    image_util::TextureData,
    render_app::AppData,
    uniform_buffer_object::GlobalUniform,
    vertexbuffer_util::{VertexData, VertexGui},
};

#[derive(Debug, Clone)]
pub struct GuiRenderObject {
    //one for each framebuffer
    pub vertex_data: Vec<VertexData<VertexGui>>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub id: TextureId,
    pub rect: Rect,
}

pub unsafe fn create_gui_descriptor_sets(
    map: &HashMap<TextureId, TextureData>,
    device: &Device,
    data: &AppData,
    texture_id: &TextureId,
) -> anyhow::Result<Vec<vk::DescriptorSet>> {
    info!("gui descriptor");
    let layouts = vec![data.gui_descriptor_layout; data.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&info)?;
    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.global_buffer[i])
            .offset(0)
            .range(size_of::<GlobalUniform>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        let image_data = map.get(texture_id).unwrap();

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_data.image_view)
            .sampler(image_data.sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
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

    Ok(descriptor_sets)
}
pub struct Gui {
    pub render_objects: Vec<GuiRenderObject>,
    pub image_map: HashMap<TextureId, TextureData>,
    // let images go through all framebuffers before removing, to all images to be removed are not being used
    pub images_to_destroy: Vec<(u8, TextureData)>,
    pub egui_state: egui_winit::State,
    ///what should be presented when running the app
    pub show: fn(&Context, &mut Ui),
}
impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Gui (
        vertex_data: {:?},
        image_map: {:?},
    )",
            self.render_objects, self.image_map
        ))
        .unwrap();
        Ok(())
    }
}

impl Gui {
    pub fn get_window_and_ctx(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> anyhow::Result<(egui::Context, window::Window)> {
        let egui_ctx = egui::Context::default();
        let viewport_builder = egui::viewport::ViewportBuilder::default()
            .with_title("Eligine")
            .with_inner_size(egui::Vec2::new(1024.0, 768.0));

        let window = egui_winit::create_window(&egui_ctx, event_loop, &viewport_builder).unwrap();
        Ok((egui_ctx, window))
    }
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        egui_ctx: egui::Context,
        window: &window::Window,
        show: fn(&Context, &mut Ui),
    ) -> anyhow::Result<Self> {
        let viewport_id = egui_ctx.viewport_id();
        let egui_state = egui_winit::State::new(
            egui_ctx,
            viewport_id,
            event_loop,
            Some(window.scale_factor() as f32),
            Some(winit::window::Theme::Dark),
            None,
        );
        Ok(Self {
            render_objects: vec![],
            image_map: HashMap::new(),
            images_to_destroy: vec![],
            egui_state,
            show,
        })
    }

    pub fn run_egui(
        &mut self,
        window: &window::Window,
        event: &winit::event::WindowEvent,
    ) -> FullOutput {
        // Each frame:
        let _response = self.egui_state.on_window_event(window, event);
        let input = self.egui_state.take_egui_input(window);

        self.egui_state.egui_ctx().begin_pass(input);
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(self.egui_state.egui_ctx(), |ui| {
                (self.show)(&self.egui_state.egui_ctx(), ui)
            });

        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }

    pub fn run_egui_fst(&mut self, window: &window::Window) -> FullOutput {
        let input = egui::RawInput::default();

        self.egui_state.egui_ctx().begin_pass(input);
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(self.egui_state.egui_ctx(), |ui| {
                (self.show)(&self.egui_state.egui_ctx(), ui)
            });
        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        for (_, data) in &mut self.image_map {
            data.destroy_image(device);
        }
        for v in &self.render_objects {
            for v in &v.vertex_data {
                device.free_memory(v.vertex_buffer_memory, None);
                device.destroy_buffer(v.vertex_buffer, None);
                device.free_memory(v.index_buffer_memory, None);
                device.destroy_buffer(v.index_buffer, None);
            }
        }
    }

    pub fn update_gui_images(
        &mut self,
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        output: &FullOutput,
    ) -> anyhow::Result<()> {
        let image_delta = &output.textures_delta;

        for (id, delta) in &image_delta.set {
            if let Some(image_data) = self.image_map.remove(id) {
                self.images_to_destroy
                    .push((data.framebuffers.len() as u8, image_data));
                //if image already exists we need to update it
            }
            let texture_data = match &delta.image {
                egui::ImageData::Color(color_image) => unsafe {
                    TextureData::create_gui_texture(
                        instance,
                        device,
                        data,
                        Vec::from(color_image.as_raw()),
                        (color_image.size[0] as u32, color_image.size[1] as u32),
                    )?
                },
            };
            let insert = self.image_map.insert(*id, texture_data);
        }

        for id in &image_delta.free {
            self.images_to_destroy.push((
                data.framebuffers.len() as u8,
                self.image_map.remove(id).unwrap(),
            ));
        }

        Ok(())
    }

    pub fn cleanup_garbage(&mut self, device: &Device) {
        self.images_to_destroy.retain_mut(|(count, data)| {
            if *count > 0 {
                *count -= 1;
                true
            } else {
                unsafe { data.destroy_image(device) };
                false
            }
        });
    }

    //if `image_index` is None then all images per flight will be updated otherwise, only the specified index will be updated
    pub fn update_gui_mesh(
        &mut self,
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        output: &FullOutput,
        pixels_per_point: f32,
        image_index: Option<usize>,
    ) -> anyhow::Result<()> {
        let texture_id: Vec<(TextureId, Rect)> = output
            .clone()
            .shapes
            .iter()
            .map(|s| (s.shape.texture_id(), s.clip_rect))
            .collect();
        let primitives = self
            .egui_state
            .egui_ctx()
            .tessellate(output.shapes.clone(), pixels_per_point);
        let mut gui_render_objects = vec![];
        for i in 0..primitives.len() {
            let prim = &primitives[i];
            let (id, rect) = texture_id[i];
            let (indices, verts) = match &prim.primitive {
                epaint::Primitive::Mesh(mesh) => {
                    let vertices: Vec<VertexGui> = mesh
                        .vertices
                        .iter()
                        .map(|v| VertexGui {
                            pos: Vec2 {
                                x: v.pos.x,
                                y: v.pos.y,
                            },
                            uv: Vec2::new(v.uv.x, v.uv.y),
                            color: U8Vec4::new(v.color.r(), v.color.g(), v.color.b(), v.color.a()),
                        })
                        .collect();
                    (mesh.indices.clone(), vertices)
                }
                epaint::Primitive::Callback(_) => todo!(),
            };
            let mut vertex_data = vec![];
            if let Some(image_index) = image_index {
                vertex_data[image_index] = unsafe {
                    VertexData::create_vertex_data(
                        instance,
                        device,
                        data,
                        verts.to_owned(),
                        indices,
                    )
                }?;
            } else {
                vertex_data.clear();
                for _ in 0..data.framebuffers.len() {
                    println!("pushing gui verts");
                    vertex_data.push(unsafe {
                        VertexData::create_vertex_data(
                            instance,
                            device,
                            data,
                            verts.to_owned(),
                            indices.clone(),
                        )
                    }?);
                }
            }
            gui_render_objects.push(GuiRenderObject {
                vertex_data,
                descriptor_sets: unsafe {
                    create_gui_descriptor_sets(&self.image_map, device, data, &id)
                }?,
                id,
                rect,
            });
        }
        self.render_objects = gui_render_objects;
        Ok(())
    }
}
pub fn prim_to_mesh(prim: ClippedPrimitive) -> (Vec<u32>, Vec<VertexData<VertexGui>>) {
    todo!()
}
pub fn show(ctx: &egui::Context, ui: &mut Ui) {
    SidePanel::new(egui::panel::Side::Left, "my panel ")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Hello egui! IM home you dummy, and so it goes lalalalla")
                    .color(egui::Color32::RED)
                    .size(28.0),
            );

            if ui
                .add(egui::Button::new("Click me").fill(Color32::YELLOW))
                .clicked()
            {
                println!("hello world");
            };
            if ui.button("battypatpat").clicked() {
                println!("goodbye");
            };
        });
}

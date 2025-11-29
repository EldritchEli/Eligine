#![allow(unsafe_op_in_unsafe_fn)]
use std::collections::HashMap;

use bevy::color::palettes::tailwind::RED_100;
use egui::{Color32, FullOutput, Rect, RichText, TextureId};
use glam::{U8Vec4, Vec2, Vec4};
use vulkanalia::{
    Device, Instance,
    vk::{self, DeviceV1_0},
};
use winit::window;

use crate::vulkan::{
    image_util::TextureData,
    render_app::AppData,
    vertexbuffer_util::{VertexData, VertexGui},
};

#[derive(Debug, Clone)]
pub struct GuiRenderObject {
    pub vertex_data: VertexData<VertexGui>,
    pub id: TextureId,
    pub rect: Rect,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

pub struct Gui {
    pub vertex_data: Vec<GuiRenderObject>,
    pub image_map: HashMap<TextureId, TextureData>,
    pub egui_state: egui_winit::State,
}
impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Gui (
        vertex_data: {:?},
        image_map: {:?},
    )",
            self.vertex_data, self.image_map
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
    pub fn init(
        device: &Device,
        data: &mut AppData,
        event_loop: &winit::event_loop::ActiveEventLoop,
        egui_ctx: egui::Context,
        window: &window::Window,
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
            vertex_data: vec![],
            image_map: HashMap::new(),
            egui_state,
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

        egui::CentralPanel::default().show(self.egui_state.egui_ctx(), |ui| {
            ui.label("Hello egui! IM home you dummy, and so it goes lalalalla");
            if ui.button("atoms").clicked() {
                println!("hello world");
            };
            if ui.button("battypatpat").clicked() {
                println!("goodbye");
            };
        });

        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }

    pub fn run_egui_fst(&mut self, window: &window::Window) -> FullOutput {
        // Each frame:
        let input = egui::RawInput::default();

        self.egui_state.egui_ctx().begin_pass(input);
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(self.egui_state.egui_ctx(), |ui| {
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

        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        for v in &mut self.vertex_data {
            for (_, data) in &mut self.image_map {
                data.destroy_image(device);
            }
            device.free_memory(v.vertex_data.vertex_buffer_memory, None);
            device.destroy_buffer(v.vertex_data.vertex_buffer, None);
            device.free_memory(v.vertex_data.index_buffer_memory, None);
            device.destroy_buffer(v.vertex_data.index_buffer, None);
        }
    }

    pub fn update_gui_images(
        &mut self,
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        output: FullOutput,
    ) -> anyhow::Result<()> {
        let image_delta = output.textures_delta;
        for (id, delta) in &image_delta.set {
            let texture_data = match &delta.image {
                egui::ImageData::Color(color_image) => unsafe {
                    TextureData::create_gui_texture(
                        instance,
                        device,
                        data,
                        Vec::from(color_image.as_raw()),
                        (color_image.size[0] as u32, color_image.size[1] as u32),
                    )
                },
            }?;
            let insert = self.image_map.insert(id.clone(), texture_data);
        }

        Ok(())
    }

    pub fn remove_freed_images(
        &mut self,
        device: &Device,
        output: FullOutput,
    ) -> anyhow::Result<()> {
        for id in &output.textures_delta.free {
            self.image_map.remove(id).map(|data| {
                unsafe { data.destroy_image(device) };
            });
        }
        Ok(())
    }
    pub fn update_gui_mesh(
        &mut self,
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        output: &FullOutput,
        pixels_per_point: f32,
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
        let mut vertices = vec![];
        for (prim, (id, rect)) in primitives.iter().zip(texture_id) {
            let vertex_data: VertexData<VertexGui> = match &prim.primitive {
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
                    //let mut indices: Vec<u32> = vec![];
                    /*  for i in 0..vertices.len() * 3 {
                        indices.push(0);
                    }*/
                    unsafe {
                        VertexData::create_vertex_data(
                            instance,
                            device,
                            data,
                            vertices,
                            mesh.indices.clone(),
                        )
                    }?
                }
                epaint::Primitive::Callback(_) => todo!(),
            };
            vertices.push(GuiRenderObject {
                vertex_data,
                id,
                rect,
                descriptor_sets: vec![],
            });
        }
        self.vertex_data = vertices;

        Ok(())
    }
}

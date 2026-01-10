#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
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
    pub vertex_data: VertexData<VertexGui>,
    pub descriptor_set: vk::DescriptorSet,
    pub id: TextureId,
    pub rect: Rect,
}

pub unsafe fn update_gui_descriptor_sets(
    descriptor_sets: &vk::DescriptorSet,
    map: &HashMap<TextureId, TextureData>,
    device: &Device,
    texture_id: &TextureId,
) -> anyhow::Result<()> {
    let image_data = map.get(texture_id).unwrap();

    let info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(image_data.image_view)
        .sampler(image_data.sampler);

    let image_info = &[info];
    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(*descriptor_sets)
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_info);

    unsafe { device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]) };
    Ok(())
}
pub unsafe fn create_gui_descriptor_sets(
    map: &HashMap<TextureId, TextureData>,
    device: &Device,
    data: &AppData,
    texture_id: &TextureId,
) -> anyhow::Result<vk::DescriptorSet> {
    info!("gui descriptor");
    let layouts = vec![data.gui_descriptor_layout; 1];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_set = *device.allocate_descriptor_sets(&info)?.first().unwrap();
    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.global_buffer[i])
            .offset(0)
            .range(size_of::<GlobalUniform>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
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
            .dst_set(descriptor_set)
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

    Ok(descriptor_set)
}
pub struct Gui {
    pub render_objects: Vec<Vec<GuiRenderObject>>,
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
            .frame(egui::Frame::new())
            .show(self.egui_state.egui_ctx(), |ui| {
                (self.show)(self.egui_state.egui_ctx(), ui)
            });

        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }

    pub fn run_egui_fst(&mut self, window: &window::Window) -> FullOutput {
        let input = self.egui_state.take_egui_input(window);

        self.egui_state.egui_ctx().begin_pass(input);
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(self.egui_state.egui_ctx(), |ui| {
                (self.show)(self.egui_state.egui_ctx(), ui)
            });
        self.egui_state.egui_ctx().end_pass()

        // handle full_output
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        for (_, data) in &mut self.image_map {
            data.destroy_image(device);
        }
        for v in &self.render_objects {
            for v in v {
                device.free_memory(v.vertex_data.vertex_buffer_memory, None);
                device.destroy_buffer(v.vertex_data.vertex_buffer, None);
                device.free_memory(v.vertex_data.index_buffer_memory, None);
                device.destroy_buffer(v.vertex_data.index_buffer, None);
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
                //for now we destroy the old instanc and create a new one
                //but we can probably figure out a smarter way to do this.
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
    pub unsafe fn update_gui_mesh(
        &mut self,
        instance: &Instance,
        device: &Device,
        data: &mut AppData,
        output: &FullOutput,
        pixels_per_point: f32,
        image_index: usize,
    ) -> anyhow::Result<()> {
        let render_objects = &mut self.render_objects[image_index];
        println!("render_object length: {:?}", render_objects.len());
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
        if primitives.len() < render_objects.len() {
            for obj in render_objects.drain(primitives.len()..) {
                device.destroy_buffer(obj.vertex_data.vertex_buffer, None);
                device.destroy_buffer(obj.vertex_data.index_buffer, None);
                device.free_memory(obj.vertex_data.vertex_buffer_memory, None);
                device.free_memory(obj.vertex_data.index_buffer_memory, None);
                device.free_descriptor_sets(data.descriptor_pool, &[obj.descriptor_set])?;
                let map = obj.vertex_data.mem_map.unwrap();

                device.destroy_buffer(map.index.staging_buffer, None);
                device.destroy_buffer(map.vertex.staging_buffer, None);
                device.free_memory(map.index.staging_memory, None);
                device.free_memory(map.vertex.staging_memory, None);
            }
        }
        for i in 0..render_objects.len() {
            let (id, rect) = texture_id[i];
            let (indices, verts) = prim_to_mesh(&primitives[i]);
            render_objects[i]
                .vertex_data
                .update_vertex_data(instance, device, data, verts, indices)?;
            render_objects[i].id = id;
            render_objects[i].rect = rect;
            device
                .free_descriptor_sets(data.descriptor_pool, &[render_objects[i].descriptor_set])?;

            render_objects[i].descriptor_set =
                create_gui_descriptor_sets(&self.image_map, device, data, &id)?;
        }

        for i in render_objects.len()..primitives.len() {
            let (id, rect) = texture_id[i];
            let (indices, verts) = prim_to_mesh(&primitives[i]);
            let vertex_data = unsafe {
                VertexData::create_vertex_data(
                    instance,
                    device,
                    data,
                    verts.to_owned(),
                    indices,
                    true,
                )
            }?;
            self.render_objects[image_index].push(GuiRenderObject {
                vertex_data,
                descriptor_set: unsafe {
                    create_gui_descriptor_sets(&self.image_map, device, data, &id)
                }?,
                id,
                rect,
            });
        }
        Ok(())
    }

    pub fn init_gui_mesh(
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
        for image_index in 0..data.framebuffers.len() {
            let mut gui_render_objects = Vec::with_capacity(primitives.len());
            for i in 0..primitives.len() {
                let prim = &primitives[i];
                let (id, rect) = texture_id[i];
                let (indices, verts) = prim_to_mesh(prim);
                let vertex_data = unsafe {
                    VertexData::create_vertex_data(
                        instance,
                        device,
                        data,
                        verts.to_owned(),
                        indices.clone(),
                        true,
                    )
                }?;
                gui_render_objects.push(GuiRenderObject {
                    vertex_data,
                    descriptor_set: unsafe {
                        create_gui_descriptor_sets(&self.image_map, device, data, &id)
                    }?,
                    id,
                    rect,
                });
            }
            self.render_objects.push(gui_render_objects)
        }
        Ok(())
    }
}
pub fn prim_to_mesh(prim: &ClippedPrimitive) -> (Vec<u32>, Vec<VertexGui>) {
    match &prim.primitive {
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
    }
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
                .add(egui::Button::new("Click me sconbbyy snack").fill(Color32::YELLOW))
                .clicked()
            {
                println!("hello world");
            };
            if ui.button("battypatpat").clicked() {
                println!("goodbye");
            };
            let alternatives = ["a", "b", "c", "d"];
            let mut selected = 2;
            egui::ComboBox::from_label("Select one!").show_index(
                ui,
                &mut selected,
                alternatives.len(),
                |i| alternatives[i],
            );
            let alternatives = ["a", "b", "cidd", "scooby snack", "ekka", "e"];
            let mut selected = 2;
            egui::ComboBox::from_label("Select two!").show_index(
                ui,
                &mut selected,
                alternatives.len(),
                |i| format!("scooby one {i}"),
            );
            egui::ComboBox::from_id_salt("my-combobox")
                .selected_text("text")
                .icon(filled_triangle)
                .show_ui(ui, |_ui| {});
        });
}
pub fn filled_triangle(
    ui: &egui::Ui,
    rect: egui::Rect,
    visuals: &egui::style::WidgetVisuals,
    _is_open: bool,
) {
    let rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width() * 0.6, rect.height() * 0.4),
    );
    ui.painter().add(egui::Shape::convex_polygon(
        vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
        visuals.fg_stroke.color,
        visuals.fg_stroke,
    ));
}

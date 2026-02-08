#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use std::{collections::HashMap, ops::Deref, sync::Arc};

use bevy::{
    core_pipeline::core_2d::graph::input,
    ecs::{entity::Entity, observer::On, query::With, system::ResMut, world::World},
    input::keyboard::{KeyCode, KeyboardInput},
    window::PrimaryWindow,
    winit::{DisplayHandleWrapper, WINIT_WINDOWS},
};
use egui::{
    ClippedPrimitive, FullOutput, Pos2, Rect, SidePanel, TextureId, TexturesDelta, Ui, ViewportInfo,
};
use egui_winit::{apply_viewport_builder_to_window, create_winit_window_attributes};
use glam::{U8Vec4, Vec2};
use itertools::Either;
use log::info;
use vulkanalia::{
    Device, Instance,
    vk::{self, DeviceV1_0, HasBuilder},
};
use winit::window;

use crate::winit_app::winit_render_app::AppData;
use crate::{
    bevy_app::render::VulkanApp,
    game_objects::scene::Scene,
    gui::{
        gui, menu,
        objects::{self, selected_object},
    },
    vulkan::{
        image_util::TextureData,
        input_state::InputState,
        uniform_buffer_object::GlobalUniform,
        vertexbuffer_util::{VertexData, VertexGui},
    },
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
    pub enabled: bool,
    pub render_objects: Vec<GuiRenderObject>,
    pub image_map: HashMap<TextureId, TextureData>,
    // let images go through all framebuffers before removing, to all images to be removed are not being used
    pub images_to_destroy: Vec<(u8, TextureData)>,
    pub egui_state: egui_winit::State,
    pub viewport_info: Option<ViewportInfo>,
    pub callback: Rect,
    pub needs_redraw: bool,
    pub output: Option<FullOutput>,
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

pub fn create_gui_from_window(world: &mut World) {
    let mut primary_window_query = world.query_filtered::<Entity, With<PrimaryWindow>>();
    let window_entity = primary_window_query.single(world).unwrap();
    let (egui_ctx, viewport_id, scale_factor) = WINIT_WINDOWS.with_borrow(|windows| {
        let window = windows.get_window(window_entity).unwrap();
        let egui_ctx = egui::Context::default();
        let viewport_builder = egui::viewport::ViewportBuilder::default()
            .with_title("Eligine")
            .with_inner_size(egui::Vec2::new(1024.0, 768.0));
        let window_attributes = create_winit_window_attributes(&egui_ctx, viewport_builder.clone());
        apply_viewport_builder_to_window(&egui_ctx, window, &viewport_builder);
        let viewport_id = egui_ctx.viewport_id();
        (egui_ctx, viewport_id, window.scale_factor())
    });

    let display_handle = world.get_resource::<DisplayHandleWrapper>().unwrap();
    let display = &display_handle.0;
    let egui_state = egui_winit::State::new(
        egui_ctx,
        viewport_id,
        display,
        Some(scale_factor as f32),
        Some(winit::window::Theme::Dark),
        None,
    );

    world.insert_non_send_resource(Gui {
        enabled: false,
        render_objects: vec![],
        image_map: HashMap::new(),
        images_to_destroy: vec![],
        egui_state,
        viewport_info: None,
        callback: Rect {
            min: Pos2::new(0.0, 0.0),
            max: Pos2::new(0.0, 0.0),
        },
        needs_redraw: true,
        output: None,
    });
}
impl Gui {
    pub fn set_enabled(&mut self, input: &mut InputState) {
        if input.f12.is_entered() {
            println!("gui enabled");
            self.enabled = !self.enabled;
        }
    }
    /// there must be an available winit_window to query. otherwise a panic will occur
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
            enabled: false,
            render_objects: vec![],
            image_map: HashMap::new(),
            images_to_destroy: vec![],
            egui_state,
            viewport_info: None,
            callback: Rect {
                min: Pos2::new(0.0, 0.0),
                max: Pos2::new(0.0, 0.0),
            },
            needs_redraw: false,
            output: None,
        })
    }

    pub fn run_egui(&mut self, data: &mut AppData, scene: &mut Scene, window: &window::Window) {
        let viewport_info = self.viewport_info.as_mut().unwrap();
        egui_winit::update_viewport_info(viewport_info, self.egui_state.egui_ctx(), window, false);
        self.output = Some(self.run_egui_fst(data, scene, window));
    }

    pub fn old_run_egui_bevy(
        &mut self,
        app: &mut VulkanApp,
        data: &mut AppData,
        scene: &mut Scene,
        window: &window::Window,
    ) {
        // Each frame:
        let input = if let Some(viewport_info) = self.viewport_info.as_mut() {
            egui_winit::update_viewport_info(
                viewport_info,
                self.egui_state.egui_ctx(),
                window,
                false,
            );
            self.egui_state.take_egui_input(window)
        } else {
            let input = self.egui_state.take_egui_input(window);
            self.viewport_info = Some(input.viewport().clone());
            input
        };

        self.output = Some(self.egui_state.egui_ctx().run(input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::new())
                .show(ctx, |ui| self.callback = show(data, scene, ctx, ui));
        }))
    }
    pub fn run_egui_bevy(
        &mut self,
        mut data: ResMut<AppData>,
        mut scene: ResMut<Scene>,
        window: &window::Window,
    ) -> FullOutput {
        let viewport_info = self.viewport_info.as_mut().unwrap();

        egui_winit::update_viewport_info(viewport_info, self.egui_state.egui_ctx(), window, false);
        self.run_egui_fst(&mut data, &mut scene, window)
    }

    pub fn run_egui_fst(
        &mut self,
        data: &mut AppData,
        scene: &mut Scene,
        window: &window::Window,
    ) -> FullOutput {
        let input = self.egui_state.take_egui_input(window);
        if self.viewport_info.is_none() {
            self.viewport_info = Some(input.viewport().clone())
        }
        self.egui_state.egui_ctx().run(input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::new())
                .show(ctx, |ui| self.callback = show(data, scene, ctx, ui));
        })

        // handle full_output
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        for (_, data) in &mut self.image_map {
            data.destroy_image(device);
        }
        for v in &self.render_objects {
            {
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
        image_delta: &TexturesDelta,
    ) -> anyhow::Result<()> {
        if !image_delta.is_empty() {
            println!("image delta is not empty");
        }
        for (id, delta) in &image_delta.set {
            if delta.is_whole() {
                println!("delta is whole");
            } else {
                println!("delta is partial")
            }

            println!("image set");
            if delta.is_whole()
                && let Some(image_data) = self.image_map.remove(id)
            {
                println!("removing old atlas");
                self.images_to_destroy
                    .push((data.framebuffers.len() as u8, image_data));
                //if image already exists we need to update it
                //for now we destroy the old instanc and create a new one
                //but we can probably figure out a smarter way to do this.
            }
            if !delta.is_whole()
                && let Some(old_image_data) = self.image_map.get(id)
                && let Some([x, y]) = delta.pos
            {
                println!("patching atlas");
                let egui::ImageData::Color(color_image) = &delta.image;
                unsafe {
                    old_image_data.patch_image(
                        instance,
                        device,
                        data,
                        Vec::from(color_image.as_raw()),
                        (color_image.size[0] as u32, color_image.size[1] as u32),
                        (x as i32, y as i32),
                    )?
                };
            } else {
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
                let _insert = self.image_map.insert(*id, texture_data);
            }
        }

        for id in &image_delta.free {
            println!("image free");
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
        if self.render_objects.is_empty() {
            return self.init_gui_mesh(instance, device, data, output, pixels_per_point);
        }
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
        if primitives.len() < self.render_objects.len() {
            for obj in self.render_objects.drain(primitives.len()..) {
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
        for i in 0..self.render_objects.len() {
            let (id, _) = texture_id[i];
            let rect = primitives[i].clip_rect;
            match prim_to_mesh(&primitives[i]) {
                Either::Left(rect) => self.callback = rect,
                Either::Right((indices, verts)) => {
                    self.render_objects[i]
                        .vertex_data
                        .update_vertex_data(instance, device, data, verts, indices)?;
                    self.render_objects[i].id = id;
                    self.render_objects[i].rect = rect;
                    update_gui_descriptor_sets(
                        &self.render_objects[i].descriptor_set,
                        &self.image_map,
                        device,
                        &id,
                    )?;
                }
            }
        }

        for i in self.render_objects.len()..primitives.len() {
            let (id, _) = texture_id[i];
            let rect = primitives[i].clip_rect;
            match prim_to_mesh(&primitives[i]) {
                Either::Left(rect) => self.callback = rect,
                Either::Right((indices, verts)) => {
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
                    self.render_objects.push(GuiRenderObject {
                        vertex_data,
                        descriptor_set: unsafe {
                            create_gui_descriptor_sets(&self.image_map, device, data, &id)
                        }?,
                        id,
                        rect,
                    });
                }
            }
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
        self.render_objects = Vec::with_capacity(primitives.len());
        for i in 0..primitives.len() {
            let prim = &primitives[i];
            let rect = prim.clip_rect;
            let (id, _) = texture_id[i];

            match prim_to_mesh(prim) {
                Either::Left(rect) => self.callback = rect,
                Either::Right((indices, verts)) => {
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
                    self.render_objects.push(GuiRenderObject {
                        vertex_data,
                        descriptor_set: unsafe {
                            create_gui_descriptor_sets(&self.image_map, device, data, &id)
                        }?,
                        id,
                        rect,
                    });
                }
            }
        }
        Ok(())
    }
}
pub fn prim_to_mesh(prim: &ClippedPrimitive) -> Either<Rect, (Vec<u32>, Vec<VertexGui>)> {
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
            Either::Right((mesh.indices.clone(), vertices))
        }
        epaint::Primitive::Callback(a) => Either::Left(a.rect),
    }
}

pub fn show(data: &mut AppData, scene: &mut Scene, ctx: &egui::Context, ui: &mut Ui) -> Rect {
    menu::show_menu(ctx);
    SidePanel::new(egui::panel::Side::Left, "my panel ")
        .default_width(200.0)
        .min_width(10.0)
        .show(ctx, |ui| {
            scene.selected_object = objects::show_objects(scene, ctx, ui);
            ui.separator();
            ui.label("Camera");
            ui.horizontal(|ui| {
                ui.label("Field of view");
                egui::DragValue::new(&mut scene.camera.fov)
            });
            ui.horizontal(|ui| {
                ui.label("Field of view");
                ui.add(egui::DragValue::new(&mut scene.camera.fov))
            });
            ui.horizontal(|ui| {
                ui.label("Near field");
                ui.add(egui::DragValue::new(&mut scene.camera.near_field).speed(0.01))
            });
            ui.horizontal(|ui| {
                ui.label("Far field");
                ui.add(egui::DragValue::new(&mut scene.camera.far_field))
            });
            ui.horizontal(|ui| {
                ui.label("slerp_speed");
                ui.add(
                    egui::DragValue::new(&mut scene.camera.slerp_speed)
                        .speed(0.01)
                        .range(0.0..=1.0),
                )
            });
            ui.horizontal(|ui| {
                ui.label("lerp_speed");
                ui.add(
                    egui::DragValue::new(&mut scene.camera.lerp_speed)
                        .speed(0.01)
                        .range(0.0..=1.0),
                )
            });
        });

    egui::TopBottomPanel::bottom("bottom panel")
        .max_height(400.0)
        .min_height(200.0)
        .show(ctx, |ui| {
            selected_object(scene, ctx, ui);
        });
    paint_callback(ctx, ui)
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

pub fn paint_callback(ctx: &egui::Context, ui: &mut Ui) -> Rect {
    let area = egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(ctx, |ui| {});
    area.response.rect
}

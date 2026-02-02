#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use crate::winit_app::winit_render_app::{self, AppData, FrameInfo};
use crate::{
    game_objects::scene::Scene,
    gui::gui::{Gui, create_gui_descriptor_sets},
    vulkan::{
        MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED,
        color_objects::create_color_objects,
        command_buffer_util::{create_command_buffer, create_command_buffers},
        command_pool::{create_command_pools, create_transient_command_pool},
        descriptor_util::{
            create_descriptor_pool, create_global_buffers, create_pbr_descriptor_sets,
            create_skybox_descriptor_sets, create_uniform_buffers, gui_descriptor_set_layout,
            pbr_descriptor_set_layout, skybox_descriptor_set_layout,
        },
        device_util::{create_logical_device, pick_physical_device},
        framebuffer_util::{create_depth_objects, create_framebuffers},
        instance_util::create_instance,
        pipeline_util::{create_pbr_pipeline, gui_pipeline, skybox_pipeline},
        render_pass_util::create_render_pass,
        swapchain_util::{create_swapchain, create_swapchain_image_views},
        sync_util::create_sync_objects,
        uniform_buffer_object::{
            GlobalUniform, OrthographicLight, PbrPushConstant, PbrUniform, UniformBuffer,
        },
        vertexbuffer_util::VertexPbr,
    },
};
use anyhow::anyhow;
use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Commands, Query},
    },
    window::PrimaryWindow,
    winit::WINIT_WINDOWS,
};
use std::intrinsics::copy_nonoverlapping as memcpy;

use egui::FullOutput;
use glam::Mat4;
use tracing::{error, info};
use vulkanalia::{
    Device, Entry, Instance,
    loader::{LIBRARY, LibloadingLoader},
    vk::{
        self, DeviceV1_0, ExtDebugUtilsExtension, Handle, HasBuilder, InstanceV1_0,
        KhrSurfaceExtension, KhrSwapchainExtension,
    },
    window as vk_window,
};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

#[derive(Debug, Resource)]
pub struct VulkanApp {
    pub entry: Entry,
    pub instance: Instance,
    //pub data: AppData,
    //pub window: Window,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub time_stamp: f32,
}
pub fn create_vulkan_resources(
    mut commands: Commands,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(primary_window.single().unwrap()) {
            let entity = primary_window.single().unwrap();
            let window = windows.get_window(entity).unwrap();
            unsafe {
                let resized = false;
                let loader = LibloadingLoader::new(LIBRARY).unwrap();
                let mut scene = Scene::default();
                let entry = Entry::new(loader)
                    .inspect_err(|_| eprintln!("failed to create entry"))
                    .unwrap();
                let mut data = AppData::default();
                let instance = create_instance(window, &entry, &mut data).unwrap();
                data.surface = vk_window::create_surface(
                    &instance,
                    &window.display_handle().unwrap(),
                    &window.window_handle().unwrap(),
                )
                .unwrap();
                pick_physical_device(&instance, &mut data).unwrap();
                let device = create_logical_device(&entry, &instance, &mut data).unwrap();
                create_swapchain(window, &instance, &device, &mut data).unwrap();
                create_swapchain_image_views(&device, &mut data).unwrap();
                create_render_pass(&instance, &device, &mut data).unwrap();
                pbr_descriptor_set_layout(&device, &mut data).unwrap();
                skybox_descriptor_set_layout(&device, &mut data).unwrap();
                gui_descriptor_set_layout(&device, &mut data).unwrap();
                gui_pipeline(&device, &mut data, 0).unwrap();
                create_pbr_pipeline(&device, &mut data, 1).unwrap();
                skybox_pipeline(&device, &mut data, 2).unwrap();
                create_color_objects(&instance, &device, &mut data).unwrap();
                create_depth_objects(&instance, &device, &mut data).unwrap();
                create_framebuffers(&device, &mut data).unwrap();
                create_command_pools(&instance, &device, &mut data).unwrap();
                create_transient_command_pool(&instance, &device, &mut data).unwrap();
                create_descriptor_pool(&device, &mut data, 30).unwrap();

                create_command_buffers(&device, &mut scene, &mut data, window, None).unwrap();
                create_sync_objects(&device, &mut data).unwrap();

                create_global_buffers(&instance, &device, &mut data, &mut scene).unwrap();

                commands.insert_resource(scene);
                commands.insert_resource(data);
                commands.insert_resource(VulkanApp {
                    resized,
                    frame: 0,
                    time_stamp: 0.0,
                    entry,
                    instance,

                    device,
                });
            }
        }
    });
}
pub fn recreate_swapchain(
    instance: &Instance,
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    window: &Window,
    gui: &mut Gui,
) {
    unsafe {
        println!("here");
        device.device_wait_idle().unwrap();
        println!("device");
        destroy_swapchain(data, device, scene);
        println!("after destroy swap");
        if let Err(e) = create_swapchain(window, instance, device, data) {
            panic!("swapchain creation error: {:?}", e);
        }
        println!("after create swap");
        create_swapchain_image_views(&device, data).unwrap();
        create_render_pass(&instance, &device, data).unwrap();

        create_descriptor_pool(&device, data, 30).unwrap();
        skybox_pipeline(&device, data, 2).unwrap();
        create_pbr_pipeline(&device, data, 1).unwrap();
        gui_pipeline(&device, data, 0).unwrap();
        println!("beree");
        create_color_objects(&instance, &device, data).unwrap();
        create_depth_objects(&instance, &device, data).unwrap();
        create_framebuffers(&device, data).unwrap();
        create_global_buffers(&instance, &device, data, scene).unwrap();
        create_skybox_descriptor_sets(&device, &data, scene).unwrap();
        for (_, object) in scene.render_objects.iter_mut() {
            create_uniform_buffers::<PbrUniform>(
                &instance,
                &device,
                data,
                &mut object.uniform_buffers,
                &mut object.uniform_buffers_memory,
            )
            .unwrap();
            create_pbr_descriptor_sets::<VertexPbr, PbrUniform>(
                &device,
                data,
                &mut scene.sun,
                object,
            )
            .unwrap();
        }
        println!("here");
        for objects in &mut gui.render_objects {
            for object in objects {
                object.descriptor_set =
                    create_gui_descriptor_sets(&gui.image_map, &device, &data, &object.id).unwrap();
            }
        }

        create_command_buffers(&device, scene, data, window, Some(gui)).unwrap();
        data.recreated = true;
        info!("recreated swapchain");
    }
}

pub fn update_uniform_buffer(
    image_index: usize,
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    window: &Window,

    gui: &mut Gui,
) {
    let view = scene.camera.transform.matrix();

    let proj = scene.camera.projection_matrix(&data, gui);

    data.pbr_push_contant = PbrPushConstant {
        proj_inv_view: (view.inverse()),
    };
    for (_i, object) in scene.render_objects.iter() {
        let mut model = [Mat4::default(); 10];
        for (index, instance_index) in object.instances.iter().enumerate() {
            let instance = scene.objects.get(*instance_index).unwrap();
            model[index] = instance.global_matrix(&scene);
        }
        let ubo = PbrUniform {
            model,
            base: object.pbr.base,
        };
        unsafe { ubo.map_memory(device, object.uniform_buffers_memory[image_index]) }.unwrap();
    }
    if scene.skybox.is_some() {
        let scale = window.scale_factor() as f32;
        let ubo = GlobalUniform {
            view,
            proj,
            x: data.swapchain_extent.width as f32 / scale,
            y: data.swapchain_extent.height as f32 / scale,
        };
        unsafe { ubo.map_memory(&device, data.global_buffer_memory[image_index]) }.unwrap();
    }
    let memory = unsafe {
        device.map_memory(
            scene.sun.memory[image_index],
            0,
            size_of::<OrthographicLight>() as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe { memcpy(&scene.sun.omnidirectional_light, memory.cast(), 1) };

    unsafe { device.unmap_memory(scene.sun.memory[image_index]) };
}

/// Renders a frame for our Vulkan app.
pub unsafe fn render(
    app: &mut VulkanApp,
    data: &mut AppData,
    scene: &mut Scene,
    window: &Window,
    gui: &mut Gui,
) {
    app.device
        .wait_for_fences(&[data.in_flight_fences[app.frame]], true, u64::MAX)
        .unwrap();

    let result = app.device.acquire_next_image_khr(
        data.swapchain,
        u64::MAX,
        data.image_available_semaphores[app.frame],
        vk::Fence::null(),
    );

    let image_index = match result {
        Ok((image_index, vk::SuccessCode::SUBOPTIMAL_KHR)) => {
            if cfg!(target_os = "macos") {
                image_index as usize
            } else {
                recreate_swapchain(&app.instance, &app.device, scene, data, window, gui);
                return;
            }
        }
        Ok((image_index, _)) => {
            //app.data.recreated = false;
            image_index as usize
        }
        Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
            recreate_swapchain(&app.instance, &app.device, scene, data, window, gui);
            return;
        }
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    };

    let image_in_flight = data.images_in_flight[image_index];
    if !image_in_flight.is_null() {
        app.device
            .wait_for_fences(&[image_in_flight], true, u64::MAX)
            .unwrap();
    }

    data.images_in_flight[image_index] = data.in_flight_fences[app.frame];
    if gui.egui_state.egui_ctx().has_requested_repaint()
        && let Some(egui_output) = gui.output.take()
    {
        gui.update_gui_images(
            &app.instance,
            &app.device,
            data,
            &egui_output.textures_delta,
        )
        .unwrap();
        gui.update_gui_mesh(
            &app.instance,
            &app.device,
            data,
            &egui_output,
            gui.egui_state.egui_ctx().pixels_per_point(),
            image_index,
        )
        .unwrap();
        gui.needs_redraw = false;
    }
    update_uniform_buffer(image_index, &app.device, scene, data, window, gui);

    let wait_semaphores = &[data.image_available_semaphores[app.frame]];
    let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    create_command_buffer(&app.device, scene, data, window, Some(gui), image_index).unwrap();
    let command_buffers = &[data.command_centers[image_index].command_buffers[0]];
    let signal_semaphores = &[data.render_finished_semaphores[app.frame]];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(wait_semaphores)
        .wait_dst_stage_mask(wait_stages)
        .command_buffers(command_buffers)
        .signal_semaphores(signal_semaphores);

    app.device
        .reset_fences(&[data.in_flight_fences[app.frame]])
        .unwrap();

    app.device
        .queue_submit(
            data.graphics_queue,
            &[submit_info],
            data.in_flight_fences[app.frame],
        )
        .unwrap();

    let swapchains = &[data.swapchain];
    let image_indices = &[image_index as u32];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(signal_semaphores)
        .swapchains(swapchains)
        .image_indices(image_indices);

    let result = app
        .device
        .queue_present_khr(data.present_queue, &present_info);
    let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
        || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

    if app.resized || changed {
        app.resized = false;
        recreate_swapchain(&app.instance, &app.device, scene, data, window, gui);
    } else if let Err(e) = result {
        error!("{e:?}");
        return;
    }
    app.device.queue_wait_idle(data.present_queue).unwrap();

    app.frame = (app.frame + 1) % MAX_FRAMES_IN_FLIGHT;
}

/// Destroys our Vulkan app.
pub(crate) unsafe fn destroy(
    app: &VulkanApp,
    data: &mut AppData,
    scene: &mut Scene,
    gui: &mut Gui,
) {
    unsafe {
        app.device.device_wait_idle().unwrap();
        destroy_swapchain(data, &app.device, scene);
        app.device
            .destroy_descriptor_set_layout(data.gui_descriptor_layout, None);

        for data in gui.image_map.values() {
            app.device.destroy_sampler(data.sampler, None);
            app.device.destroy_image_view(data.image_view, None);

            app.device.destroy_image(data.image, None);
            app.device.free_memory(data.image_memory, None);
        }
        for (_i, object) in scene.render_objects.iter() {
            app.device
                .destroy_sampler(object.pbr.texture_data.sampler, None);
            app.device
                .destroy_image_view(object.pbr.texture_data.image_view, None);

            app.device
                .destroy_image(object.pbr.texture_data.image, None);
            app.device
                .free_memory(object.pbr.texture_data.image_memory, None);
        }

        app.device
            .destroy_descriptor_set_layout(data.pbr_descriptor_set_layout, None);
        app.device
            .destroy_descriptor_set_layout(data.skybox_descriptor_set_layout, None);
        data.in_flight_fences
            .iter()
            .for_each(|f| app.device.destroy_fence(*f, None));
        data.render_finished_semaphores
            .iter()
            .for_each(|s| app.device.destroy_semaphore(*s, None));
        data.image_available_semaphores
            .iter()
            .for_each(|s| app.device.destroy_semaphore(*s, None));
        for objects in &gui.render_objects {
            for object in objects {
                app.device
                    .free_memory(object.vertex_data.vertex_buffer_memory, None);
                app.device
                    .destroy_buffer(object.vertex_data.vertex_buffer, None);
                app.device
                    .free_memory(object.vertex_data.index_buffer_memory, None);
                app.device
                    .destroy_buffer(object.vertex_data.index_buffer, None);
                if let Some(staging_map) = &object.vertex_data.mem_map {
                    app.device
                        .free_memory(staging_map.index.staging_memory, None);
                    app.device
                        .destroy_buffer(staging_map.index.staging_buffer, None);
                    app.device
                        .free_memory(staging_map.vertex.staging_memory, None);
                    app.device
                        .destroy_buffer(staging_map.vertex.staging_buffer, None);
                }
            }
        }
        for (_i, object) in scene.render_objects.iter() {
            app.device
                .free_memory(object.vertex_data.vertex_buffer_memory, None);
            app.device
                .destroy_buffer(object.vertex_data.vertex_buffer, None);
            app.device
                .free_memory(object.vertex_data.index_buffer_memory, None);
            app.device
                .destroy_buffer(object.vertex_data.index_buffer, None);
        }
        if let Some(skybox) = &scene.skybox {
            skybox.texture_data.destroy_image(&app.device)
        }
        for center in &data.command_centers {
            app.device.destroy_command_pool(center.command_pool, None);
        }
        app.device.destroy_command_pool(data.single_time_pool, None);
        app.device
            .destroy_command_pool(data.transient_command_pool, None);
        app.device.destroy_device(None);
        app.instance.destroy_surface_khr(data.surface, None);
        if VALIDATION_ENABLED {
            app.instance
                .destroy_debug_utils_messenger_ext(data.messenger, None);
        }
        app.instance.destroy_instance(None);
    }
}

fn destroy_swapchain(data: &mut AppData, device: &Device, scene: &mut Scene) {
    unsafe {
        device.destroy_image_view(data.color_image_view, None);
        device.free_memory(data.color_image_memory, None);
        device.destroy_image(data.color_image, None);

        device.destroy_image_view(data.depth_image_view, None);
        device.free_memory(data.depth_image_memory, None);
        device.destroy_image(data.depth_image, None);
        device.destroy_descriptor_pool(data.descriptor_pool, None);
        scene.render_objects.iter().for_each(|(_i, object)| {
            object
                .uniform_buffers
                .iter()
                .for_each(|b| device.destroy_buffer(*b, None));
            object
                .uniform_buffers_memory
                .iter()
                .for_each(|m| device.free_memory(*m, None));
        });
        data.global_buffer
            .iter()
            .for_each(|b| device.destroy_buffer(*b, None));
        data.global_buffer_memory
            .iter()
            .for_each(|m| device.free_memory(*m, None));
        scene
            .sun
            .buffer
            .iter()
            .for_each(|b| device.destroy_buffer(*b, None));
        scene
            .sun
            .memory
            .iter()
            .for_each(|m| device.free_memory(*m, None));
        /*self.data
            .uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data
            .uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        */
        data.framebuffers
            .iter()
            .for_each(|f| device.destroy_framebuffer(*f, None));
        for center in &data.command_centers {
            device.free_command_buffers(center.command_pool, &center.command_buffers);
        }
        device.destroy_pipeline(data.pbr_pipeline, None);
        device.destroy_pipeline_layout(data.pbr_pipeline_layout, None);
        device.destroy_pipeline(data.skybox_pipeline, None);
        device.destroy_pipeline_layout(data.skybox_pipeline_layout, None);

        device.destroy_pipeline(data.gui_pipeline, None);
        device.destroy_pipeline_layout(data.gui_pipeline_layout, None);

        device.destroy_render_pass(data.render_pass, None);
        data.swapchain_image_views
            .iter()
            .for_each(|v| device.destroy_image_view(*v, None));
        device.destroy_swapchain_khr(data.swapchain, None);
    }
}

use std::intrinsics::copy_nonoverlapping as memcpy;

use crate::{
    game_objects::scene::Scene,
    gui::gui::{Gui, create_gui_descriptor_sets},
    vulkan::{
        MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED,
        color_objects::create_color_objects,
        command_buffer_util::{create_command_buffer, create_command_buffers},
        descriptor_util::{
            create_descriptor_pool, create_global_buffers, create_pbr_descriptor_sets,
            create_skybox_descriptor_sets, create_uniform_buffers,
        },
        framebuffer_util::{create_depth_objects, create_framebuffers},
        pipeline_util::{create_pbr_pipeline, gui_pipeline, skybox_pipeline},
        render_app::{self, AppData, FrameInfo},
        render_pass_util::create_render_pass,
        swapchain_util::{create_swapchain, create_swapchain_image_views},
        uniform_buffer_object::{
            GlobalUniform, OrthographicLight, PbrPushConstant, PbrUniform, UniformBuffer,
        },
        vertexbuffer_util::VertexPbr,
    },
};

use egui::FullOutput;
use glam::Mat4;
use tracing::{error, info};
use vulkanalia::{
    Device, Instance,
    vk::{
        self, DeviceV1_0, ExtDebugUtilsExtension, Handle, HasBuilder, InstanceV1_0,
        KhrSurfaceExtension, KhrSwapchainExtension,
    },
};
use winit::window::Window;

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
pub fn render(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    frame_info: &mut FrameInfo,
    window: &Window,
    gui: &mut Gui,
    egui_output: Option<FullOutput>,
) {
    unsafe {
        device
            .wait_for_fences(&[data.in_flight_fences[frame_info.frame]], true, u64::MAX)
            .unwrap();

        let result = device.acquire_next_image_khr(
            data.swapchain,
            u64::MAX,
            data.image_available_semaphores[frame_info.frame],
            vk::Fence::null(),
        );
        //let sem = data.image_available_semaphores[frame];

        let image_index = match result {
            Ok((image_index, vk::SuccessCode::SUBOPTIMAL_KHR)) => {
                if cfg!(target_os = "macos") {
                    image_index as usize
                } else {
                    return recreate_swapchain(instance, device, scene, data, window, gui);
                }
            }
            Ok((image_index, _)) => {
                //self.data.recreated = false;
                image_index as usize
            }
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                return recreate_swapchain(instance, device, scene, data, window, gui);
            }
            Err(e) => {
                error!("alien success code: {:?}", e);
                return;
            }
        };

        let image_in_flight = data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            device
                .wait_for_fences(&[image_in_flight], true, u64::MAX)
                .unwrap();
        }

        data.images_in_flight[image_index] = data.in_flight_fences[frame_info.frame];
        gui.cleanup_garbage(&device);
        println!("before egui");
        if let Some(egui_output) = egui_output {
            println!("in egui");
            gui.update_gui_images(&instance, &device, data, &egui_output.textures_delta)
                .unwrap();
            gui.update_gui_mesh(
                &instance,
                &device,
                data,
                &egui_output,
                gui.egui_state.egui_ctx().pixels_per_point(),
                image_index,
            )
            .unwrap();
        }
        println!("after egui");
        update_uniform_buffer(image_index, device, scene, data, window, gui);

        println!("after update uniform");
        let wait_semaphores = &[data.image_available_semaphores[frame_info.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        create_command_buffer(&device, scene, data, window, Some(gui), image_index).unwrap();
        let command_buffers = &[data.command_centers[image_index].command_buffers[0]];
        let signal_semaphores = &[data.render_finished_semaphores[frame_info.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        device
            .reset_fences(&[data.in_flight_fences[frame_info.frame]])
            .unwrap();

        device
            .queue_submit(
                data.graphics_queue,
                &[submit_info],
                data.in_flight_fences[frame_info.frame],
            )
            .unwrap();

        let swapchains = &[data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);
        println!("before present");
        let result = device.queue_present_khr(data.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if frame_info.resized || changed {
            frame_info.resized = false;
            println!("before recreate");
            recreate_swapchain(instance, device, scene, data, window, gui);
        } else if let Err(e) = result {
            return error!("queue present error: {:?}", e);
        }
        println!("before queue idle");
        device.queue_wait_idle(data.present_queue).unwrap();

        frame_info.frame = (frame_info.frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

/// Destroys our Vulkan app.
pub(crate) unsafe fn destroy(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    gui: &mut Gui,
) {
    unsafe {
        device.device_wait_idle().unwrap();
        destroy_swapchain(data, device, scene);
        device.destroy_descriptor_set_layout(data.gui_descriptor_layout, None);

        for data in gui.image_map.values() {
            device.destroy_sampler(data.sampler, None);
            device.destroy_image_view(data.image_view, None);

            device.destroy_image(data.image, None);
            device.free_memory(data.image_memory, None);
        }
        for (_i, object) in scene.render_objects.iter() {
            device.destroy_sampler(object.pbr.texture_data.sampler, None);
            device.destroy_image_view(object.pbr.texture_data.image_view, None);

            device.destroy_image(object.pbr.texture_data.image, None);
            device.free_memory(object.pbr.texture_data.image_memory, None);
        }

        device.destroy_descriptor_set_layout(data.pbr_descriptor_set_layout, None);
        device.destroy_descriptor_set_layout(data.skybox_descriptor_set_layout, None);
        data.in_flight_fences
            .iter()
            .for_each(|f| device.destroy_fence(*f, None));
        data.render_finished_semaphores
            .iter()
            .for_each(|s| device.destroy_semaphore(*s, None));
        data.image_available_semaphores
            .iter()
            .for_each(|s| device.destroy_semaphore(*s, None));
        for objects in &gui.render_objects {
            for object in objects {
                device.free_memory(object.vertex_data.vertex_buffer_memory, None);
                device.destroy_buffer(object.vertex_data.vertex_buffer, None);
                device.free_memory(object.vertex_data.index_buffer_memory, None);
                device.destroy_buffer(object.vertex_data.index_buffer, None);
                if let Some(staging_map) = &object.vertex_data.mem_map {
                    device.free_memory(staging_map.index.staging_memory, None);
                    device.destroy_buffer(staging_map.index.staging_buffer, None);
                    device.free_memory(staging_map.vertex.staging_memory, None);
                    device.destroy_buffer(staging_map.vertex.staging_buffer, None);
                }
            }
        }
        for (_i, object) in scene.render_objects.iter() {
            device.free_memory(object.vertex_data.vertex_buffer_memory, None);
            device.destroy_buffer(object.vertex_data.vertex_buffer, None);
            device.free_memory(object.vertex_data.index_buffer_memory, None);
            device.destroy_buffer(object.vertex_data.index_buffer, None);
        }
        if let Some(skybox) = &scene.skybox {
            skybox.texture_data.destroy_image(&device);
        }
        for center in &data.command_centers {
            device.destroy_command_pool(center.command_pool, None);
        }
        device.destroy_command_pool(data.single_time_pool, None);
        device.destroy_command_pool(data.transient_command_pool, None);
        device.destroy_device(None);
        instance.destroy_surface_khr(data.surface, None);
        if VALIDATION_ENABLED {
            instance.destroy_debug_utils_messenger_ext(data.messenger, None);
        }
        instance.destroy_instance(None);
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

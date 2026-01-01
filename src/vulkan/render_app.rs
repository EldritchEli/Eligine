#![allow(unsafe_op_in_unsafe_fn)]
use crate::gui::gui::Gui;
use crate::vulkan::command_buffer_util::create_command_buffers;
use crate::vulkan::command_pool::{create_command_pool, create_transient_command_pool};
use crate::vulkan::descriptor_util::{
    create_descriptor_pool, create_global_buffers, create_gui_descriptor_sets,
    create_pbr_descriptor_sets, create_skybox_descriptor_sets, create_uniform_buffers,
    gui_descriptor_set_layout, pbr_descriptor_set_layout, skybox_descriptor_set_layout,
};
use crate::vulkan::device_util::{create_logical_device, pick_physical_device};
use crate::vulkan::framebuffer_util::{create_depth_objects, create_framebuffers};
use crate::vulkan::instance_util::create_instance;
use crate::vulkan::pipeline_util::{create_pbr_pipeline, gui_pipeline, skybox_pipeline};
use crate::vulkan::render_pass_util::create_render_pass;
use crate::vulkan::swapchain_util::{create_swapchain, create_swapchain_image_views};
use crate::vulkan::sync_util::create_sync_objects;
use crate::vulkan::uniform_buffer_object::{
    GlobalUniform, OrthographicLight, PbrUniform, UniformBuffer,
};
use crate::vulkan::vertexbuffer_util::VertexPbr;
use crate::vulkan::{CORRECTION, FAR_PLANE_DISTANCE, MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED};
use crate::{gltf, gui};
use anyhow::anyhow;
use std::f32::consts::PI;
use std::path::Path;
use std::time::Instant;
use vulkanalia::loader::{LIBRARY, LibloadingLoader};
use vulkanalia::vk::{
    DeviceV1_0, ExtDebugUtilsExtension, Handle, HasBuilder, InstanceV1_0, KhrSurfaceExtension,
    KhrSwapchainExtension,
};
use vulkanalia::window as vk_window;
use vulkanalia::{Device, Entry, Instance, vk};
use winit::window::Window;

use crate::game_objects::render_object::ObjectId;
use crate::game_objects::scene::Scene;
use crate::vulkan::color_objects::create_color_objects;
use glam::Mat4;
use std::ptr::copy_nonoverlapping as memcpy;

/// Our Vulkan app.
#[derive(Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    //pub window: Window,
    pub scene: Scene,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub surface: vk::SurfaceKHR,
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub msaa_samples: vk::SampleCountFlags,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    pub render_pass: vk::RenderPass,
    pub pbr_descriptor_set_layout: vk::DescriptorSetLayout,
    pub skybox_descriptor_set_layout: vk::DescriptorSetLayout,
    pub pbr_pipeline_layout: vk::PipelineLayout,
    pub pbr_pipeline: vk::Pipeline,
    pub skybox_pipeline_layout: vk::PipelineLayout,
    pub skybox_pipeline: vk::Pipeline,
    pub gui_descriptor_layout: vk::DescriptorSetLayout,
    pub gui_pipeline_layout: vk::PipelineLayout,
    pub gui_pipeline: vk::Pipeline,
    pub global_buffer: Vec<vk::Buffer>,
    pub global_buffer_memory: Vec<vk::DeviceMemory>,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub command_pool: vk::CommandPool,
    pub transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub transient_command_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    pub descriptor_pool: vk::DescriptorPool,

    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,

    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
}
impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> anyhow::Result<Self> {
        let resized = false;
        let loader = LibloadingLoader::new(LIBRARY)?;
        let mut scene = Scene::default();
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, window, window)?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        let start = Instant::now();
        //let x = window.inner_size().width as f32;
        //let y = window.inner_size().height as f32;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        pbr_descriptor_set_layout(&device, &mut data)?;
        skybox_descriptor_set_layout(&device, &mut data)?;
        gui_descriptor_set_layout(&device, &mut data)?;
        create_pbr_pipeline(&device, &mut data, 1)?;
        gui_pipeline(&device, &mut data, 0)?;
        skybox_pipeline(&device, &mut data, 2)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_color_objects(&instance, &device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        create_transient_command_pool(&instance, &device, &mut data)?;
        create_descriptor_pool(&device, &mut data, 30)?;

        create_command_buffers(&device, &mut scene, &mut data, window, None)?;
        create_sync_objects(&device, &mut data)?;

        create_global_buffers(&instance, &device, &mut data, &mut scene)?;
        let app = Self {
            entry,
            instance,
            data,
            scene,

            device,
            frame: 0,
            resized,
            start,
            //window,
        };
        Ok(app)
    }

    pub fn add_object(&mut self, path: impl AsRef<Path>) -> Result<Vec<ObjectId>, ()> {
        let object_id = match gltf::load::scene(
            &self.instance,
            &self.device,
            &mut self.data,
            &mut &mut self.scene,
            path.as_ref(),
        ) {
            Ok(object) => object,
            Err(e) => panic!("you fucked up {:?}", e),
        };
        Ok(object_id)
    }

    pub fn add_objects(&mut self, paths: &[&str]) -> Vec<ObjectId> {
        let mut game_object_ids = vec![];
        for path in paths {
            if let Ok(gs) = self.add_object(path) {
                gs.into_iter().for_each(|g| game_object_ids.push(g));
            }
        }
        game_object_ids
    }

    unsafe fn recreate_swapchain(&mut self, window: &Window, gui: &mut Gui) -> anyhow::Result<()> {
        println!("recreated swap");
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        //mock_render_pass(&self.instance, &self.device, &mut self.data)?;

        skybox_pipeline(&self.device, &mut self.data, 2)?;
        create_pbr_pipeline(&self.device, &mut self.data, 1)?;
        gui_pipeline(&self.device, &mut self.data, 0)?;

        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        //create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data, 30)?;
        create_global_buffers(
            &self.instance,
            &self.device,
            &mut self.data,
            &mut self.scene,
        )?;
        create_skybox_descriptor_sets(&self.device, &self.data, &mut self.scene)?;
        for (_, object) in self.scene.render_objects.iter_mut() {
            create_uniform_buffers::<PbrUniform>(
                &self.instance,
                &self.device,
                &mut self.data,
                &mut object.uniform_buffers,
                &mut object.uniform_buffers_memory,
            )?;
            create_pbr_descriptor_sets::<VertexPbr, PbrUniform>(
                &self.device,
                &mut self.data,
                &mut self.scene.sun,
                object,
            )?;
        }
        for v in &mut gui.vertex_data {
            create_gui_descriptor_sets(&gui.image_map, &self.device, &self.data, v)?;
        }

        create_command_buffers(
            &self.device,
            &mut self.scene,
            &mut self.data,
            window,
            Some(gui),
        )?;
        Ok(())
    }

    pub unsafe fn update_uniform_buffer(
        &mut self,
        image_index: usize,
        window: &Window,
    ) -> anyhow::Result<()> {
        let _time = self.start.elapsed().as_secs_f32();
        //let gui = &mut self.gui.as_mut().unwrap();
        let view = self.scene.camera.transform.matrix();

        let aspect =
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32;
        let perspective = Mat4::perspective_rh(PI / 4.0, aspect, 0.1, FAR_PLANE_DISTANCE);
        let proj = CORRECTION * perspective;

        //let model_rotation: Mat4 = Mat4::from_rotation_y(PI / 4.0 * time);
        for (_i, object) in self.scene.render_objects.iter() {
            let mut model = [Mat4::default(); 10];
            for (index, instance_index) in object.instances.iter().enumerate() {
                let instance = self.scene.objects.get(*instance_index).unwrap();
                model[index] = instance.global_matrix(&self.scene);
            }
            let ubo = PbrUniform {
                model,
                base: object.pbr.base,
            };
            ubo.map_memory(&self.device, object.uniform_buffers_memory[image_index])?;
        }
        if let Some(_) = &self.scene.skybox {
            let scale = window.scale_factor() as f32;
            let ubo = GlobalUniform {
                view,
                proj,
                x: self.data.swapchain_extent.width as f32 / scale,
                y: self.data.swapchain_extent.height as f32 / scale,
            };
            ubo.map_memory(&self.device, self.data.global_buffer_memory[image_index])?;
        }
        let memory = unsafe {
            self.device.map_memory(
                self.scene.sun.memory[image_index],
                0,
                size_of::<OrthographicLight>() as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;
        memcpy(&self.scene.sun.omnidirectional_light, memory.cast(), 1);

        self.device.unmap_memory(self.scene.sun.memory[image_index]);
        Ok(())
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window, gui: &mut Gui) -> anyhow::Result<()> {
        self.device
            .wait_for_fences(&[self.data.in_flight_fences[self.frame]], true, u64::MAX)?;

        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );
        //let sem = self.data.image_available_semaphores[self.frame];

        let image_index = match result {
            Ok((_, vk::SuccessCode::SUBOPTIMAL_KHR)) => {
                return self.recreate_swapchain(window, gui);
            }
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window, gui),
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }

        self.data.images_in_flight[image_index] = self.data.in_flight_fences[self.frame];

        self.update_uniform_buffer(image_index, window)?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device
            .reset_fences(&[self.data.in_flight_fences[self.frame]])?;

        self.device.queue_submit(
            self.data.graphics_queue,
            &[submit_info],
            self.data.in_flight_fences[self.frame],
        )?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.data.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window, gui)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }
        self.device.queue_wait_idle(self.data.present_queue)?;

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub(crate) unsafe fn destroy(&mut self, gui: &mut Gui) {
        self.device.device_wait_idle().unwrap();
        self.destroy_swapchain();
        self.device
            .destroy_descriptor_set_layout(self.data.gui_descriptor_layout, None);

        for (_i, data) in &gui.image_map {
            self.device.destroy_sampler(data.sampler, None);
            self.device.destroy_image_view(data.image_view, None);

            self.device.destroy_image(data.image, None);
            self.device.free_memory(data.image_memory, None);
        }
        // self.gui.as_mut().unwrap().destroy(&self.device);
        for (_i, object) in self.scene.render_objects.iter() {
            self.device
                .destroy_sampler(object.pbr.texture_data.sampler, None);
            self.device
                .destroy_image_view(object.pbr.texture_data.image_view, None);

            self.device
                .destroy_image(object.pbr.texture_data.image, None);
            self.device
                .free_memory(object.pbr.texture_data.image_memory, None);
        }

        self.device
            .destroy_descriptor_set_layout(self.data.pbr_descriptor_set_layout, None);
        self.device
            .destroy_descriptor_set_layout(self.data.skybox_descriptor_set_layout, None);
        self.data
            .in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data
            .render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data
            .image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        for v in &gui.vertex_data {
            self.device
                .free_memory(v.vertex_data.vertex_buffer_memory, None);
            self.device
                .destroy_buffer(v.vertex_data.vertex_buffer, None);
            self.device
                .free_memory(v.vertex_data.index_buffer_memory, None);
            self.device.destroy_buffer(v.vertex_data.index_buffer, None);
        }
        for (_i, object) in self.scene.render_objects.iter() {
            self.device
                .free_memory(object.vertex_data.vertex_buffer_memory, None);
            self.device
                .destroy_buffer(object.vertex_data.vertex_buffer, None);
            self.device
                .free_memory(object.vertex_data.index_buffer_memory, None);
            self.device
                .destroy_buffer(object.vertex_data.index_buffer, None);
        }
        if let Some(skybox) = &self.scene.skybox {
            skybox.texture_data.destroy_image(&self.device);
        }

        self.device
            .destroy_command_pool(self.data.command_pool, None);
        self.device
            .destroy_command_pool(self.data.transient_command_pool, None);

        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);
        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
        self.instance.destroy_instance(None);
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.device
            .destroy_image_view(self.data.color_image_view, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image(self.data.color_image, None);

        self.device
            .destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);
        self.device
            .destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.scene.render_objects.iter().for_each(|(_i, object)| {
            object
                .uniform_buffers
                .iter()
                .for_each(|b| self.device.destroy_buffer(*b, None));
            object
                .uniform_buffers_memory
                .iter()
                .for_each(|m| self.device.free_memory(*m, None));
        });
        self.data
            .global_buffer
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data
            .global_buffer_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        self.scene
            .sun
            .buffer
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.scene
            .sun
            .memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        /*self.data
            .uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data
            .uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        */
        self.data
            .framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device
            .free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.device.destroy_pipeline(self.data.pbr_pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.pbr_pipeline_layout, None);
        self.device
            .destroy_pipeline(self.data.skybox_pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.skybox_pipeline_layout, None);

        self.device.destroy_pipeline(self.data.gui_pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.gui_pipeline_layout, None);

        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}

#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod buffer_util;
mod color_objects;
mod command_buffer_util;
mod command_pool;
mod descriptor_util;
mod device_util;
mod framebuffer_util;
mod game_objects;
mod image_util;
mod input_state;
mod instance_util;
mod pipeline_util;
mod queue_family_indices;
mod render_app;
mod render_pass_util;
mod shader_module_util;
mod swapchain_util;
mod sync_util;
mod uniform_buffer_object;
mod varlen;
mod vertexbuffer_util;


use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::game_objects::scene::Scene;
use crate::input_state::InputState;
use crate::queue_family_indices::QueueFamilyIndices;
use crate::render_app::{App, AppData};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    let mut input_state = InputState::new();
    // Window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Eligine")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;
    let mut time_stamp = 0.0;
    // App

    let mut app = unsafe { App::create(&window)? };


    let mut minimized = false; //window minimize

    event_loop.run(move |event, elwt| {
        let elapsed = app.start.elapsed().as_secs_f32();
        let dt = elapsed - time_stamp;

        time_stamp = elapsed;
        input_state.read_event(&event);
        app.scene.update(dt, &input_state);
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => app.resized = true,
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => {
                    unsafe { app.render(&window) }.unwrap()
                }
                WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        minimized = true;
                    } else {
                        minimized = false;
                        app.resized = true;
                    }
                }
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    unsafe {
                        app.device.device_wait_idle().unwrap();
                    }
                    unsafe {
                        app.destroy();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    })?;
    Ok(())
}

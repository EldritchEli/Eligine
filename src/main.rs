#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod swapchain_util;
mod device_util;
mod render_app;
mod pipeline_util;
mod instance_util;
mod queue_family_indices;
mod command_buffer_util;
mod render_pass_util;
mod sync_util;
mod command_pool;
mod framebuffer_util;
mod shader_module_util;
mod vertexbuffer_util;
mod buffer_util;
mod descriptor_util;
mod transforms;
mod image_util;
mod varlen;
mod color_objects;
mod input_state;
mod game_objects;



use anyhow::{Result};
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{WindowBuilder};

use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use crate::game_objects::Scene::Scene;
use crate::input_state::InputState;
use crate::queue_family_indices::QueueFamilyIndices;
use crate::render_app::{App, AppData};



const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_ENABLED: bool =
    cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let mut input_state = InputState::new();
    // Window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Eligine")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    let mut minimized = false; //window minimize

    event_loop.run(move |event, elwt| {

        input_state.read_event(&event);
        app.scene.update(0.0001, &input_state);
        match event {
            // Request a redraw when all events were processed.

            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => app.resized = true,
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => unsafe { app.render(&window) }.unwrap(),
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
                    unsafe { app.device.device_wait_idle().unwrap(); }
                    unsafe { app.destroy(); }
                }
                _ => {}
            }
            _ => {}
        }
    })?;
    Ok(())
}


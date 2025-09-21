#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use crate::vulkan::input_state::InputState;
use crate::vulkan::render_app::App;
use anyhow::Result;
use terrors::OneOf;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ErrorCode;
use winit::dpi::LogicalSize;
use winit::error::{EventLoopError, OsError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct VulkanData {
    input_state: InputState,
    event_loop: EventLoop<()>,
    window: Window,
    window_minimized: bool,
    time_stamp: f32,
    app: App,
}
pub fn init(
    window_name: &str,
) -> Result<VulkanData, OneOf<(OsError, EventLoopError, anyhow::Error)>> {
    pretty_env_logger::init();
    let input_state = InputState::new();
    // Window
    let event_loop = EventLoop::new().map_err(|e| OneOf::new(e))?;
    let window = WindowBuilder::new()
        .with_title("Eligine")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .map_err(|e| OneOf::new(e))?;
    let app = unsafe { App::create(&window).map_err(|e| OneOf::new(e))? };
    let time_stamp = 0.0;
    let window_minimized = false; //window minimize
    Ok(VulkanData {
        input_state,
        event_loop,
        window,
        app,
        window_minimized,
        time_stamp,
    })
}

pub fn run(
    vulkan_data: VulkanData,
) -> Result<(), OneOf<(anyhow::Error, ErrorCode, EventLoopError)>> {
    let VulkanData {
        mut input_state,
        event_loop,
        window,
        mut window_minimized,
        mut time_stamp,
        mut app,
    } = vulkan_data;
    event_loop
        .run(move |event, elwt| {
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
                    WindowEvent::RedrawRequested if !elwt.exiting() && !window_minimized => {
                        unsafe { app.render(&window) }.unwrap()
                    }
                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            window_minimized = true;
                        } else {
                            window_minimized = false;
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
        })
        .map_err(|e| OneOf::new(e))?;
    Ok(())
}

fn main() -> Result<(), OneOf<(OsError, anyhow::Error, EventLoopError, ErrorCode)>> {
    let vulkan_data = init("Eligine").map_err(OneOf::broaden)?;
    run(vulkan_data).map_err(OneOf::broaden)
}

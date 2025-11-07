#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]
use crate::game_objects::scene::Scene;
use crate::vulkan::input_state::InputState;
use crate::vulkan::render_app::App;
use anyhow::Result;

use log::error;
use terrors::OneOf;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ErrorCode;
use vulkanalia::window;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::{EventLoopError, OsError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{self, ActiveEventLoop};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;
use winit::window::{Window, WindowAttributes};

pub enum AppState {
    Uninitialized { scene: Scene },
    Initialized { app: App },
}
impl AppState {
    pub fn initialized(&self) -> bool {
        match self {
            AppState::Uninitialized { scene } => false,
            AppState::Initialized { app } => true,
        }
    }
    pub fn set_window(&mut self, window: Window) {
        match self {
            AppState::Uninitialized { scene } => (),
            AppState::Initialized { app } => app.window = window,
        }
    }
    pub fn request_redraw(&mut self) {
        match self {
            AppState::Uninitialized { scene } => todo!(),
            AppState::Initialized { app } => app.window.request_redraw(),
        }
    }
}
pub struct VulkanData {
    pub input_state: InputState,
    pub window_minimized: bool,
    pub window_name: String,
    pub time_stamp: f32,
    pub render_stamp: f32,
    pub app: AppState,
}

impl Default for VulkanData {
    fn default() -> Self {
        Self {
            input_state: Default::default(),
            window_name: "Eligine".to_string(),
            window_minimized: false,
            time_stamp: 0.0,
            render_stamp : 0.0,
            app: AppState::Uninitialized {
                scene: Scene::default(),
            },
        }
    }
}

impl ApplicationHandler for VulkanData {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attr = WindowAttributes::default()
            .with_title("Eligine")
            .with_inner_size(LogicalSize::new(1024, 768));
        //.build(&event_loop)

        let window = event_loop.create_window(attr).unwrap();
        window.request_redraw();
        if self.app.initialized() {
            self.app.set_window(window);
        } else {
            self.app = AppState::Initialized {
                app: unsafe { App::create(window).unwrap() },
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = match &mut self.app {
            AppState::Uninitialized { scene } => {
                error!("uninitialized app");
                return;
            }
            AppState::Initialized { app } => app,
        };

       let elapsed = app.start.elapsed().as_secs_f32();
        let mut dt = elapsed - self.time_stamp;
        
        self.time_stamp = elapsed;
        self.input_state.read_event(&event);
        app.scene.update(dt, &self.input_state);
       // print!("window event");
        match event {
            // Request a redraw when all events were processed.
            /* WindowEvent::AboutToWait => self.window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => app.resized = true,*/
            // Render a frame if our Vulkan app is not being destroyed.
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.window_minimized = true;
                } else {
                    self.window_minimized = false;
                    app.resized = true;
                }
            }
            // Destroy our Vulkan app.
            WindowEvent::CloseRequested => {
                event_loop.exit();
                unsafe {
                    app.device.device_wait_idle().unwrap();
                }
                unsafe {
                    app.destroy();
                }
            }
            WindowEvent::RedrawRequested => {
        
        //let elapsed = app.start.elapsed().as_secs_f32();
            
        app.window.request_redraw();
      /*   if elapsed - self.render_stamp > (1.0 / 60.0) {
            self.render_stamp = elapsed;*/
            unsafe { app.render() }.unwrap()
       // }
            }
            _ => 
            /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */{
            }

        }
    }
}
/*
pub fn init(
    window_name: &str,
) -> Result<VulkanData, OneOf<(OsError, EventLoopError, anyhow::Error)>> {
    pretty_env_logger::init();
    let input_state = InputState::new();
    //let event_loop = ActiveEventLoop::create_window(&self, WindowAttributes::)
    // Window
    let event_loop = EventLoop::new().map_err(|e| OneOf::new(e))?;
    let window = event_loop
        .create_window(
            WindowAttributes::new()
                .with_title("Eligine")
                .with_inner_size(LogicalSize::new(1024, 768)), //.build(&event_loop)
        )
        .unwrap();
    let app = unsafe { App::create(&window).map_err(|e| OneOf::new(e))? };
    let time_stamp = 0.0;
    let window_minimized = false; //window minimize
    Ok(VulkanData {
        input_state,
        event_loop,
        app: Some(app),
        window_minimized,
        time_stamp,
    })
}
*/
/*
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
*/

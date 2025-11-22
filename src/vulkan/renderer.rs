#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    unsafe_op_in_unsafe_fn
)]
use crate::vulkan::input_state::InputState;
use crate::vulkan::render_app::App;

//use egui_winit_vulkano::{Gui, GuiConfig};
use log::error;
use vulkanalia::prelude::v1_0::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::window::{Window, WindowAttributes};

pub enum AppState {
    Uninitialized { init: fn(&mut App) },
    Initialized { app: App },
}

impl AppState {
    pub fn initialized(&self) -> bool {
        match self {
            AppState::Uninitialized { .. } => false,
            AppState::Initialized { app } => true,
        }
    }

    pub fn set_window(&mut self, window: Window) {
        match self {
            AppState::Uninitialized { .. } => (),
            AppState::Initialized { app } => app.window = window,
        }
    }
    pub fn request_redraw(&mut self) {
        match self {
            AppState::Uninitialized { .. } => todo!(),
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
    //pub gui: Option<Gui>,
}
impl VulkanData {
    pub fn set_init(&mut self, init: fn(&mut App)) -> Result<(), String> {
        if let AppState::Initialized { .. } = self.app {
            return Err("cannot set init for already initialized app".to_string());
        }
        self.app = AppState::Uninitialized { init };
        Ok(())
    }

    pub fn run_init(&mut self, window: Window) -> Result<(), String> {
        match &self.app {
            AppState::Initialized { app } => Err("already initialized".to_string()),
            AppState::Uninitialized { init } => {
                let mut app = unsafe { App::create(window).unwrap() };

                init(&mut app);

                self.app = AppState::Initialized { app: app };
                Ok(())
            }
        }
    }
}

impl Default for VulkanData {
    fn default() -> Self {
        Self {
            input_state: Default::default(),
            window_name: "Eligine".to_string(),
            window_minimized: false,
            time_stamp: 0.0,
            render_stamp: 0.0,
            app: AppState::Uninitialized { init: |app| {} },
            //gui: None,
        }
    }
}

impl ApplicationHandler for VulkanData {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attr = WindowAttributes::default()
            .with_title("Eligine")
            .with_inner_size(LogicalSize::new(1024, 768));
        let window = event_loop.create_window(attr).unwrap();
        window.request_redraw();
        if self.app.initialized() {
            self.app.set_window(window);
        } else {
            if let Err(st) = self.run_init(window) {
                error!("{:?}", st);
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
            AppState::Uninitialized { .. } => {
                error!("uninitialized app");
                return;
            }
            AppState::Initialized { app } => app,
        };

        let elapsed = app.start.elapsed().as_secs_f32();
        let dt = elapsed - self.time_stamp;

        self.time_stamp = elapsed;
        self.input_state.read_event(&event);
        app.scene.update(dt, &self.input_state);
        match event {
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.window_minimized = true;
                } else {
                    self.window_minimized = false;
                    app.resized = true;
                }
            }

            WindowEvent::Destroyed => {
                println!("window destroyed");
                event_loop.exit();
                unsafe {
                    app.device.device_wait_idle().unwrap();
                }
                unsafe {
                    app.destroy();
                }
            }
            // Destroy our Vulkan app.
            WindowEvent::CloseRequested => {
                println!("window closed");

                unsafe {
                    app.destroy();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app.window.request_redraw();
                unsafe { app.render() }.unwrap()
                // }
            }
            _ =>
                /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */
                {}
        }
    }
}

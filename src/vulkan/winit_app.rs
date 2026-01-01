#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    unsafe_op_in_unsafe_fn
)]
use crate::gui::gui::Gui;
use crate::vulkan::input_state::InputState;
use crate::vulkan::render_app::App;
use anyhow::anyhow;

//use egui_winit_vulkano::{Gui, GuiConfig};
use log::error;
use vulkanalia::prelude::v1_0::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowId;

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

    /*   pub fn set_window(&mut self, window: Window) {
    match self {
        AppState::Uninitialized { .. } => (),
        AppState::Initialized { app } => self.window = window,
    }*/

    /*  pub fn request_redraw(&mut self) {
        match self {
            AppState::Uninitialized { .. } => todo!(),
            AppState::Initialized { app } => app.window.request_redraw(),
        }
    }*/
}
pub struct VulkanData {
    pub input_state: InputState,
    pub gui: Option<Gui>,
    pub window: Option<Window>,
    pub window_minimized: bool,
    pub window_name: String,
    pub time_stamp: f32,
    pub render_stamp: f32,
    pub app: AppState,
}
impl VulkanData {
    pub fn set_init(&mut self, init: fn(&mut App)) -> Result<(), String> {
        if let AppState::Initialized { .. } = self.app {
            return Err("cannot set init for already initialized app".to_string());
        }
        self.app = AppState::Uninitialized { init };
        Ok(())
    }

    pub fn run_init(
        &mut self,
        window: Window,
        event_loop: &ActiveEventLoop,
        gui_ctx: Option<egui::Context>,
    ) -> anyhow::Result<()> {
        match &self.app {
            AppState::Initialized { app } => Err(anyhow!("already initialized".to_string())),
            AppState::Uninitialized { init } => {
                let mut app = unsafe { App::create(&window).unwrap() };
                init(&mut app);
                if let Some(ctx) = gui_ctx {
                    let pixels_per_point = ctx.pixels_per_point();
                    let mut gui = Gui::init(&app.device, &mut app.data, event_loop, ctx, &window)?;
                    let output = gui.run_egui_fst(&window);

                    gui.update_gui_mesh(
                        &app.instance,
                        &app.device,
                        &mut app.data,
                        &output,
                        pixels_per_point,
                    )?;
                    gui.update_gui_images(&app.instance, &app.device, &mut app.data, output)?;
                    self.gui = Some(gui);
                }
                self.app = AppState::Initialized { app: app };
                self.window = Some(window);
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
            gui: None,
            window: None,
            //gui: None,
        }
    }
}

impl ApplicationHandler for VulkanData {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (gui_ctx, window) = Gui::get_window_and_ctx(event_loop).unwrap();
        window.request_redraw();
        //self.window = window;
        if self.app.initialized() {
            // self.app.set_window(window);
        } else {
            if let Err(st) = self.run_init(window, event_loop, Some(gui_ctx)) {
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
        let gui = self.gui.as_mut().unwrap();
        let output = gui.run_egui(self.window.as_ref().unwrap(), &event);
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
                    app.destroy(self.gui.as_mut().unwrap());
                }
            }
            // Destroy our Vulkan app.
            WindowEvent::CloseRequested => {
                println!("window closed");

                unsafe {
                    app.destroy(self.gui.as_mut().unwrap());
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let window = &self.window.as_ref().unwrap();
                window.request_redraw();
                unsafe { app.render(window, self.gui.as_mut().unwrap()) }.unwrap()
            }
            _ =>
                /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */
                {}
        }
    }
}

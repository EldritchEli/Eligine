#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    unsafe_op_in_unsafe_fn
)]
use crate::gui::gui::Gui;
use crate::vulkan::input_state::InputState;
use crate::winit_app::winit_render_app::App;
use anyhow::anyhow;

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
}
impl AppState {
    pub fn unwrap(&self) -> &App {
        match self {
            AppState::Uninitialized { init } => {
                panic!("you cannot call unwrap on an uninitialized Appstate")
            }
            AppState::Initialized { app } => app,
        }
    }
    pub fn unwrap_mut(&mut self) -> &mut App {
        match self {
            AppState::Uninitialized { init } => {
                panic!("you cannot call unwrap on an uninitialized Appstate")
            }
            AppState::Initialized { app } => app,
        }
    }
}
pub struct WinitWrapper {
    pub input_state: InputState,
    pub gui: Option<Gui>,
    pub window: Option<Window>,
    pub window_minimized: bool,
    pub window_name: String,
    pub app: AppState,
}
impl WinitWrapper {
    pub fn set_init_closure(&mut self, init: fn(&mut App)) -> Result<(), String> {
        if let AppState::Initialized { .. } = self.app {
            return Err("cannot set init for already initialized app".to_string());
        }
        self.app = AppState::Uninitialized { init };
        Ok(())
    }

    pub fn init(
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
                    let mut gui = Gui::new(event_loop, ctx, &window)?;
                    let output = gui.run_egui_fst(&mut app.data, &mut app.scene, &window);
                    gui.update_gui_images(
                        &app.instance,
                        &app.device,
                        &mut app.data,
                        &output.textures_delta,
                    )?;
                    gui.init_gui_mesh(
                        &app.instance,
                        &app.device,
                        &mut app.data,
                        &output,
                        pixels_per_point,
                    )?;
                    self.gui = Some(gui);
                }
                self.app = AppState::Initialized { app };
                self.window = Some(window);
                Ok(())
            }
        }
    }
}

impl Default for WinitWrapper {
    fn default() -> Self {
        Self {
            input_state: Default::default(),
            window_name: "Eligine".to_string(),
            window_minimized: false,
            app: AppState::Uninitialized { init: |app| {} },
            gui: None,
            window: None,
        }
    }
}

impl ApplicationHandler for WinitWrapper {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (gui_ctx, window) = Gui::get_window_and_ctx(event_loop).unwrap();
        window.request_redraw();
        if self.app.initialized() {
        } else if let Err(st) = self.init(window, event_loop, Some(gui_ctx)) {
            error!("{:?}", st);
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
        let dt = elapsed - app.time_stamp;

        app.time_stamp = elapsed;
        let gui = self.gui.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();
        let response = gui.egui_state.on_window_event(window, &event);
        self.input_state.read_event(&event);
        gui.set_enabled(&mut self.input_state);
        app.scene.update(dt, &self.input_state);
        self.input_state.reset_mouse_delta();
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
                if gui.enabled {
                    gui.run_egui(&mut app.data, &mut app.scene, window);
                } else {
                };
                /*if !output.textures_delta.is_empty() {
                    gui.new_texture_delta.push(output.textures_delta.clone())
                }*/
                unsafe { app.render(window, self.gui.as_mut().unwrap()) }.unwrap();
                window.request_redraw();
            }
            _ =>
                /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */
                {}
        }
    }
}

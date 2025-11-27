use egui::Vec2;
use vulkanalia::vk::ExtPipelinePropertiesExtension;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{self, ControlFlow},
    window::WindowAttributes,
};

#[derive(Default)]
pub struct Gui {
    pub window: Option<winit::window::Window>,
    pub egui_state: Option<egui_winit::State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let egui_ctx = egui::Context::default();
        let viewport_builder = egui::viewport::ViewportBuilder::default()
            .with_title("Eligine")
            .with_inner_size(Vec2::new(1024.0, 768.0));

        let window = egui_winit::create_window(&egui_ctx, event_loop, &viewport_builder).unwrap();

        let viewport_id = egui_ctx.viewport_id();
        let state = egui_winit::State::new(
            egui_ctx,
            viewport_id,
            event_loop,
            Some(window.scale_factor() as f32),
            Some(winit::window::Theme::Dark),
            None,
        );
        self.egui_state = Some(state);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                // Each frame:
                let input = egui::RawInput::default();

                self.egui_state
                    .as_ref()
                    .unwrap()
                    .egui_ctx()
                    .begin_pass(input);

                egui::CentralPanel::default().show(
                    &self.egui_state.as_ref().unwrap().egui_ctx(),
                    |ui| {
                        ui.label("Hello egui!");
                    },
                );

                let full_output = self.egui_state.as_ref().unwrap().egui_ctx().end_pass();
                full_output.textures_delta
                // handle full_output
            }

            WindowEvent::Destroyed => {}
            // Destroy our Vulkan app.
            WindowEvent::CloseRequested => {}
            WindowEvent::RedrawRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
impl App {
    /// Returns the pixels per point of the window of this gui.
    fn pixels_per_point(&self) -> f32 {
        egui_winit::pixels_per_point(
            &self.egui_state.as_ref().unwrap().egui_ctx(),
            self.window.as_ref().unwrap(),
        )
    }
}

pub fn main() {
    let event_loop = event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

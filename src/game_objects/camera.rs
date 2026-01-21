use crate::game_objects::transform::Transform;
use crate::gui::gui::Gui;
use crate::vulkan::input_state::InputState;
use crate::vulkan::render_app::AppData;
use crate::vulkan::{CORRECTION, FAR_PLANE_DISTANCE};
use glam::{Mat4, Quat, Vec3};
use std::cmp::PartialEq;
use std::f32::consts::PI;

#[derive(PartialEq, Clone, Debug)]
pub struct Camera {
    pub movement_speed: f32,
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub fov: f32,
    pub near_field: f32,
    pub far_field: f32,
    pub transform: Transform,
}

impl Camera {
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        movement_speed: f32,
        rotation_speed: f32,
        zoom_speed: f32,
        fov: f32,
        near_field: f32,
        far_field: f32,
    ) -> Self {
        let transform = Transform {
            position,
            rotation: Quat::look_at_rh(position, look_at, -Vec3::Y),
            ..Default::default()
        };
        Self {
            movement_speed,
            rotation_speed,
            zoom_speed,
            fov,
            transform,
            near_field,
            far_field,
        }
    }

    pub fn rotate_xy(&mut self, y: f32, x: f32) {
        self.transform.rotation = Quat::from_rotation_y(self.rotation_speed * y)
            * self.transform.rotation
            * Quat::from_rotation_x(self.rotation_speed * x);
    }
    pub fn move_forward(&mut self, amount: f32) {
        let v: Vec3 = (-amount * self.movement_speed) * (self.transform.rotation * Vec3::Z);
        self.transform.position += v;
    }
    pub fn move_right(&mut self, amount: f32) {
        let v: Vec3 = (-amount * self.movement_speed) * (self.transform.rotation * Vec3::X);
        self.transform.position += v;
    }
    pub fn move_up(&mut self, amount: f32) {
        let v: Vec3 = (amount * self.movement_speed) * (self.transform.rotation * Vec3::Y);
        self.transform.position += v;
    }

    pub fn update(&mut self, delta_time: f32, input: &InputState) {
        //println!("camera position: {:?}", self.transform.position);
        let mouse_delta = 100.0 * delta_time * input.mouse_delta;
        let mult = if input.key_shift.is_down() { 2.0 } else { 1.0 };
        if input.key_w.is_down() {
            self.move_forward(delta_time * mult);
        };
        if input.key_s.is_down() {
            self.move_forward(-delta_time * mult);
        };

        if input.key_a.is_down() {
            self.move_right(delta_time * mult);
        };
        if input.key_d.is_down() {
            self.move_right(-delta_time * mult);
        };
        if input.key_e.is_down() {
            self.move_up(-delta_time * mult);
        };
        if input.key_q.is_down() {
            self.move_up(delta_time * mult);
        };

        if input.mouse_right.is_down() {
            self.rotate_xy(mouse_delta.x, mouse_delta.y);
        }
    }

    pub fn projection_matrix(&self, data: &AppData, gui: &Gui) -> Mat4 {
        let aspect = if gui.enabled {
            (gui.callback.max.x - gui.callback.min.x) / (gui.callback.max.y - gui.callback.min.y)
        } else {
            data.swapchain_extent.width as f32 / data.swapchain_extent.height as f32
        };
        CORRECTION
            * Mat4::perspective_rh(
                self.fov * (PI / 180.0),
                aspect,
                self.near_field,
                self.far_field,
            )
    }
}
impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Vec3::ZERO,
            Vec3::Z,
            4.0,
            0.025,
            1.0,
            45.0,
            0.1,
            FAR_PLANE_DISTANCE,
        )
    }
}

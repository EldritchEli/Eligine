use crate::game_objects::transform::Transform;
use crate::input_state::InputState;
use glam::{Mat4, Quat, Vec3};
use std::cmp::PartialEq;
#[derive(PartialEq, Clone, Debug)]
pub struct Camera {
    movement_speed: f32,
    rotation_speed: f32,
    zoom_speed: f32,
    fov: f32,
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
    ) -> Self {
        let transform = Transform {
            position,
            rotation: Quat::look_at_rh(position, look_at, Vec3::Y),
            ..Default::default()
        };
        Self {
            movement_speed,
            rotation_speed,
            zoom_speed,
            fov,
            transform,
        }
    }

    pub fn rotate_xy(&mut self, y: f32, x: f32) {
        self.transform.rotation = Quat::from_rotation_y(0.01 * self.rotation_speed * y)
            * self.transform.rotation
            * Quat::from_rotation_x(0.01 * self.rotation_speed * x);
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
    pub fn matrix(&self) -> Mat4 {
        let position = self.transform.position;

        Mat4::from_translation(self.transform.position) * Mat4::from_quat(self.transform.rotation)
    }

    pub fn update(&mut self, delta_time: f32, input: &InputState) {
        let mouse_delta = input.mouse_delta;

        if input.keyW.is_down() {
            self.move_forward(delta_time);
        };
        if input.keyS.is_down() {
            self.move_forward(-delta_time);
        };

        if input.keyA.is_down() {
            self.move_right(delta_time);
        };
        if input.keyD.is_down() {
            self.move_right(-delta_time);
        };
        if input.keyE.is_down() {
            self.move_up(-delta_time);
        };
        if input.keyQ.is_down() {
            self.move_up(delta_time);
        };

        if input.mouse_right.is_down() {
            self.rotate_xy(mouse_delta.x, mouse_delta.y);
        }
    }
}
impl Default for Camera {
    fn default() -> Self {
        let pos = Vec3::new(0.0, 2.0, 0.0);
        Self::new(Vec3::ZERO, Vec3::Z, 1.0, 0.20, 1.0, 45.0)
    }
}

use bevy::ecs::lifecycle::Add;
use bevy::ecs::message::MessageReader;
use bevy::ecs::observer::On;
use bevy::ecs::system::{NonSendMut, Res, ResMut};
use bevy::input::ButtonInput;
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion};
use bevy::reflect::Enum;
use bevy::time::Time;
use glam::{Mat4, Quat, Vec3};
use winit::event::MouseScrollDelta;

use crate::game_objects::transform::{self, Transform};
use crate::gui::gui::Gui;
use crate::vulkan::input_state::InputState;
use crate::vulkan::winit_render_app::{self, AppData};
use crate::vulkan::{CORRECTION, FAR_PLANE_DISTANCE};
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
    pub target_rotation: Quat,
    pub target_position: Vec3,
    pub slerp_speed: f32,
    pub lerp_speed: f32,
}

impl Camera {
    pub fn move_to_target(&mut self) {
        self.transform.position = self
            .transform
            .position
            .lerp(self.target_position, self.lerp_speed);
    }
    pub fn rotate_to_target(&mut self) {
        self.transform.rotation = self
            .transform
            .rotation
            .slerp(self.target_rotation, self.slerp_speed)
    }
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        movement_speed: f32,
        rotation_speed: f32,
        zoom_speed: f32,
        fov: f32,
        near_field: f32,
        far_field: f32,
        slerp_speed: f32,
        lerp_speed: f32,
    ) -> Self {
        let rotation = Quat::look_at_rh(position, look_at, -Vec3::Y);
        let transform = Transform {
            position,
            rotation,
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
            target_rotation: rotation,
            target_position: position,
            slerp_speed,
            lerp_speed,
        }
    }

    pub fn rotate_xy(&mut self, y: f32, x: f32) {
        self.target_rotation = Quat::from_rotation_y(self.rotation_speed * y)
            * self.target_rotation
            * Quat::from_rotation_x(self.rotation_speed * x);
    }
    pub fn move_forward(&mut self, amount: f32) {
        let v: Vec3 = (-amount * self.movement_speed) * (self.transform.rotation * Vec3::Z);
        self.target_position += v;
    }
    pub fn move_right(&mut self, amount: f32) {
        let v: Vec3 = (-amount * self.movement_speed) * (self.transform.rotation * Vec3::X);
        self.target_position += v;
    }
    pub fn move_up(&mut self, amount: f32) {
        let v: Vec3 = (amount * self.movement_speed) * (self.transform.rotation * Vec3::Y);
        self.target_position += v;
    }

    pub fn update(&mut self, delta_time: f32, input: &InputState) {
        //println!("camera position: {:?}", self.transform.position);
        let mouse_delta = 100.0 * delta_time * input.mouse_delta;
        let mult = if input.key_shift.is_down() { 3.0 } else { 1.0 };
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
        self.rotate_to_target();
        self.move_to_target();
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
            0.6,
            0.6,
        )
    }
}

pub fn update_camera_and_gui(
    mut app: ResMut<winit_render_app::App>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_delta: MessageReader<MouseMotion>,
    mut gui: NonSendMut<Gui>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    let mult = if keys.pressed(KeyCode::ShiftLeft) {
        3.0
    } else {
        1.0
    };
    let camera = &mut app.scene.camera;
    if keys.pressed(KeyCode::KeyW) {
        camera.move_forward(delta_time * mult);
    };
    if keys.pressed(KeyCode::KeyS) {
        camera.move_forward(-delta_time * mult);
    };

    if keys.pressed(KeyCode::KeyA) {
        camera.move_right(delta_time * mult);
    };
    if keys.pressed(KeyCode::KeyD) {
        camera.move_right(-delta_time * mult);
    };
    if keys.pressed(KeyCode::KeyE) {
        camera.move_up(-delta_time * mult);
    };
    if keys.pressed(KeyCode::KeyQ) {
        camera.move_up(delta_time * mult);
    };

    if mouse_buttons.pressed(MouseButton::Right) {
        let mut x = 0.0;
        let mut y = 0.0;
        for m in mouse_delta.read() {
            x += m.delta.x;
            y += m.delta.y;
        }
        camera.rotate_xy(x, y);
    }

    if keys.pressed(KeyCode::F12) {
        gui.enabled = !gui.enabled
    }
}

use std::collections::HashMap;
use crate::game_objects::camera::Camera;
use crate::input_state::InputState;
use glam::Vec3;
use std::f32::consts::PI;

use tobj::Material;
use uuid::Uuid;
use crate::game_objects::render_object::RenderObject;
use crate::game_objects::transform::Transform;
use terrors::OneOf;
#[derive(Clone, Debug)]
pub struct Scene {
    pub(crate) camera: Camera,

    materials: Vec<Material>,
}
impl Scene {



    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let camera = Camera::new(
            Vec3::new(0.0, -1.0, -3.0),
            Vec3::ZERO,
            2.5,
            0.3,
            0.0,
            PI / 4.0,
        );
        Self {
            camera,

            materials: Vec::default(),
        }
    }
}

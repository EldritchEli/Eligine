use crate::game_objects::camera::Camera;
use crate::input_state::InputState;
use glam::Vec3;
use std::f32::consts::PI;
use tobj::Material;

#[derive(Clone, Debug)]
pub struct Scene {
    pub(crate) camera: Camera,
    drawable_objects: Vec<tobj::Model>,
    materials: Vec<Material>,
}
impl Scene {
    pub fn load_object(path: &str) -> Scene {
        todo!()
    }
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
            drawable_objects: Vec::default(),
            materials: Vec::default(),
        }
    }
}

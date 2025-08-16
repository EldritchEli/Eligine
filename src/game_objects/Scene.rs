use crate::game_objects::camera::Camera;
use tobj::{LoadOptions,Material};
use tobj::Model;
use crate::input_state::InputState;
use crate::vertexbuffer_util::load_model;

#[derive(Clone, Debug)]
pub struct Scene{
    pub(crate) camera: Camera,
    drawable_objects : Vec<tobj::Model>,
    materials : Vec<Material>,
}
impl Scene{

    pub fn load_object(path: &str) -> Scene{
        todo!()
    }
    /// updates the scene by a deltatime value
    pub fn update(&mut self, delta: f32, input : &InputState){
        self.camera.update(delta, input);
    }
}

impl Default for Scene{
    fn default() -> Self {Self {camera : Camera::default(), drawable_objects : Vec::default(), materials : Vec::default()}}
}





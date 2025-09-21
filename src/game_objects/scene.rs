use crate::game_objects;
use crate::game_objects::camera::Camera;
use crate::vulkan::input_state::InputState;
use glam::Vec3;
use slab::Slab;

use std::f32::consts::PI;

use crate::game_objects::render_object::RenderObject;
use crate::game_objects::transform::Transform;
use terrors::OneOf;
use tobj::Material;
use uuid::Uuid;
#[derive(Clone, Debug)]
pub struct GameObject {
    pub transform: Transform,
    pub children: Vec<GameObject>,
    pub render_object: usize,
}

impl GameObject {}
#[derive(Clone, Debug)]
pub struct Scene {
    pub(crate) camera: Camera,
    pub render_objects: Slab<RenderObject>,
    pub objects: Slab<GameObject>,
    materials: Vec<Material>,
}
impl Scene {
    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
    }

    pub fn insert_instance(
        &mut self,
        object: GameObject,
        render_object_id: usize,
    ) -> Option<usize> {
        let render_object = self.render_objects.get_mut(render_object_id)?;
        let instance_id = self.objects.insert(object);
        render_object.instances.push(instance_id);
        Some(instance_id)
    }

    pub fn insert_from_transform(
        &mut self,
        transform: Transform,
        render_object_id: usize,
    ) -> Option<usize> {
        self.insert_instance(
            GameObject {
                transform,
                children: vec![],
                render_object: render_object_id,
            },
            render_object_id,
        )
    }
    pub fn remove_instance(&mut self, id: usize) -> Option<GameObject> {
        let object = self.objects.remove(id);
        let render = self.render_objects.get_mut(object.render_object)?;
        let index = render.instances.iter().position(|i| *i == id).unwrap();
        let _ind = render.instances.swap_remove(index);
        Some(object)
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
            render_objects: Slab::new(),
            objects: Slab::new(),
            materials: Vec::default(),
        }
    }
}

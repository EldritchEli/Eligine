use crate::game_objects::camera::Camera;
use crate::game_objects::skybox::SkyBox;
use crate::vulkan::input_state::InputState;
use crate::vulkan::uniform_buffer_object::OrthographicLight;
use crate::vulkan::vertexbuffer_util::VertexPbr;
use glam::{Mat4, Vec3};
use slab::{IntoIter, Iter, IterMut, Slab};
use vulkanalia::vk::{self};

use std::f32::consts::PI;
use std::marker::PhantomData;

use crate::game_objects::render_object::{ObjectId, RenderId, RenderObject};
use crate::game_objects::transform::Transform;
use tobj::Material;

pub trait IsId: Copy {
    fn get_id(&self) -> usize;
    fn new(u: usize) -> Self;
}
impl IsId for RenderId {
    fn get_id(&self) -> usize {
        self.0
    }

    fn new(u: usize) -> Self {
        Self(u)
    }
}
impl IsId for ObjectId {
    fn get_id(&self) -> usize {
        self.0
    }

    fn new(u: usize) -> Self {
        Self(u)
    }
}

#[derive(Debug)]
pub struct ParaSlab<Id, O>
where
    Id: IsId,
{
    slab: Slab<O>,
    phantom: PhantomData<Id>,
}

impl<Id, O> Iterator for ParaSlab<Id, O>
where
    Id: IsId,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<Id, O> ParaSlab<Id, O>
where
    Id: IsId,
{
    pub fn new() -> Self {
        ParaSlab {
            slab: Slab::new(),
            phantom: PhantomData,
        }
    }
    pub fn iter(&self) -> Iter<'_, O> {
        self.slab.iter()
    }
    pub fn into_iter(self) -> IntoIter<O> {
        self.slab.into_iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, O> {
        self.slab.iter_mut()
    }
    pub fn insert(&mut self, o: O) -> Id {
        Id::new(self.slab.insert(o))
    }
    pub fn get(&self, id: Id) -> Option<&O> {
        self.slab.get(id.get_id())
    }
    pub fn get_mut(&mut self, id: Id) -> Option<&mut O> {
        self.slab.get_mut(id.get_id())
    }
    pub fn remove(&mut self, id: Id) -> O {
        self.slab.remove(id.get_id())
    }
}
type RenderSlab = ParaSlab<RenderId, RenderObject<VertexPbr>>;
type ObjectSlab = ParaSlab<ObjectId, GameObject>;

#[derive(Debug)]
pub struct GameObject {
    pub transform: Transform,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
    pub render_objects: Vec<RenderId>,
}
///returns the global transform of the object in matrix form
impl GameObject {
    pub fn global_matrix(&self, scene: &Scene) -> Mat4 {
        let Some(id) = self.parent else {
            return self.transform.matrix();
        };
        let Some(parent) = scene.objects.get(id) else {
            panic!("none empty parent id should always be valid")
        };
        parent.global_matrix(scene) * self.transform.matrix()
    }
}
#[derive(Debug)]
pub struct Scene {
    pub(crate) camera: Camera,
    pub render_objects: RenderSlab,
    pub objects: ObjectSlab,
    materials: Vec<Material>,
    pub skybox: Option<SkyBox>,
    pub sun: Sun,
}
impl Scene {
    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
    }

    pub fn insert_instance(&mut self, object: GameObject) -> Option<ObjectId> {
        let rids = object.render_objects.clone();
        let instance_id = self.objects.insert(object);
        for render_object_id in rids {
            let render_object = self.render_objects.get_mut(render_object_id)?;
            render_object.instances.push(instance_id.clone());
        }
        Some(instance_id)
    }

    pub fn insert_from_transform(
        &mut self,
        transform: Transform,
        render_object_ids: &Vec<RenderId>,
    ) -> Option<ObjectId> {
        self.insert_instance(GameObject {
            transform,
            children: vec![],
            render_objects: render_object_ids.clone(),
            parent: None,
        })
    }
    pub fn remove_instance(&mut self, id: ObjectId) -> Option<GameObject> {
        let object = self.objects.remove(id);
        for render_id in &object.render_objects {
            // remove object reference from render_object
            let render = self.render_objects.get_mut(*render_id)?;
            let index = render.instances.iter().position(|i| i.0 == id.0).unwrap();
            let _ind = render.instances.swap_remove(index);
        }
        Some(object)
    }

    pub fn transform_object(&mut self, id: ObjectId, transform: Transform) -> Result<(), String> {
        if let Some(instance) = self.objects.get_mut(id) {
            instance.transform.position += transform.position;
            instance.transform.rotation *= transform.rotation;
            instance.transform.scale *= transform.scale;
            Ok(())
        } else {
            Err("no object found".to_string())
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            render_objects: ParaSlab::new(),
            objects: ParaSlab::new(),
            materials: Vec::default(),
            skybox: None,
            sun: Sun::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sun {
    pub omnidirectional_light: OrthographicLight,
    pub buffer: Vec<vk::Buffer>,
    pub memory: Vec<vk::DeviceMemory>,
}

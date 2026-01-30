use slab::{IntoIter, Iter, IterMut, Slab};
use std::marker::PhantomData;

use crate::{
    game_objects::{
        game_objects::{GameObject, ObjectId},
        render_object::RenderObject,
        transform::Transform,
    },
    vulkan::vertexbuffer_util::Vertex,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PbrId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AnimatedPbrId(pub usize);

pub trait IsId: Copy + PartialEq + Eq {
    fn get_id(&self) -> usize;
    fn new(u: usize) -> Self;
}
pub trait RenderId: IsId {}
impl IsId for AnimatedPbrId {
    fn get_id(&self) -> usize {
        self.0
    }

    fn new(u: usize) -> Self {
        Self(u)
    }
}
impl IsId for PbrId {
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

pub type RenderSlab<R, V>
where
    R: RenderId,
= ParaSlab<R, RenderObject<V>>;
pub type ObjectSlab<Oid, Rid, V> = ParaSlab<Oid, GameObject<Oid, Rid, V>>;

pub fn insert_instance<Oid, Rid, V>(
    renders: &mut RenderSlab<Oid, Rid, V>,
    objects: &mut ObjectSlab<Oid, Rid, V>,
    object: GameObject<Oid, Rid, V>,
) -> Option<Oid>
where
    Oid: IsId,
    Rid: RenderId,
    V: Vertex,
{
    let rids = object.render_objects.clone();
    let instance_id = objects.insert(object);
    for render_object_id in rids {
        let render_object = renders.get_mut(render_object_id)?;
        render_object.instances.push(instance_id.clone());
    }
    Some(instance_id)
}

pub fn insert_from_transform<Oid, Rid, V>(
    renders: &mut RenderSlab<Oid, Rid, V>,
    objects: &mut ObjectSlab<Oid, Rid, V>,
    transform: Transform,
    render_object_ids: &Vec<Rid>,
) -> Option<Oid>
where
    Oid: IsId,
    Rid: RenderId,
    V: Vertex,
{
    insert_instance(
        renders,
        objects,
        GameObject {
            name: "no name".into(),
            transform,
            children: vec![],
            render_objects: render_object_ids.clone(),
            parent: None,
            phantom_render: PhantomData,
        },
    )
}
pub fn remove_instance<Oid, Rid, V>(
    objects: &mut ObjectSlab<Oid, Rid, V>,
    renders: &mut RenderSlab<Oid, Rid, V>,
    id: Oid,
) -> Option<GameObject<Oid, Rid, V>>
where
    Oid: IsId,
    Rid: RenderId,
    V: Vertex,
{
    let object = objects.remove(id);
    for render_id in &object.render_objects {
        let render = renders.get_mut(*render_id)?;

        let index = render.instances.iter().position(|i| *i == id).unwrap();
        let _ind = render.instances.swap_remove(index);
    }
    Some(object)
}

pub fn transform_object<Oid, Rid, V>(
    objects: &mut ObjectSlab<Oid, Rid, V>,
    id: Oid,
    transform: Transform,
) -> Result<(), String>
where
    Oid: IsId,
    Rid: RenderId,
    V: Vertex,
{
    if let Some(instance) = objects.get_mut(id) {
        instance.transform.position += transform.position;
        instance.transform.rotation *= transform.rotation;
        instance.transform.scale *= transform.scale;
        Ok(())
    } else {
        Err("no object found".to_string())
    }
}

use std::marker::PhantomData;

use glam::Mat4;

use crate::{
    game_objects::{
        scene::{IsId, ParaSlab},
        transform::Transform,
    },
    vulkan::vertexbuffer_util::Vertex,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectId(pub usize);
#[derive(Debug)]
pub struct GameObject<ObjectId, RenderId, V>
where
    ObjectId: IsId,
    RenderId: IsId,
    V: Vertex,
{
    pub name: String,
    pub transform: Transform,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
    pub render_objects: Vec<RenderId>,
    pub phantom_render: PhantomData<V>,
}
///returns the global transform of the object in matrix form
impl<Oid, Rid, V> GameObject<Oid, Rid, V>
where
    Oid: IsId,
    Rid: RenderId,
    V: Vertex,
{
    pub fn global_matrix(&self, objects: &ParaSlab<Oid, GameObject<Oid, Rid, V>>) -> Mat4 {
        let Some(id) = self.parent else {
            return self.transform.matrix();
        };
        let Some(parent) = objects.get(id) else {
            panic!("none empty parent id should always be valid")
        };
        parent.global_matrix(objects) * self.transform.matrix()
    }
}

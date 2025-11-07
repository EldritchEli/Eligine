use crate::{
    game_objects::{
        render_object::{ObjectId, RenderId, RenderObject},
        scene::{GameObject, Scene},
        transform::Transform,
    },
    vulkan::{
        image_util::TextureData,
        render_app::AppData,
        vertexbuffer_util::{Vertex, VertexData},
    },
};

use glam::{Vec2, Vec3};
use gltf::json::accessor::{ComponentType, Type};
use gltf::{
    self,
    buffer::{self},
    image, Accessor, Node, Semantic,
};
use log::info;
use std::{collections::HashMap, f32::consts::PI, path::Path};
use terrors::OneOf;
use vulkanalia::{Device, Instance};

///loads a single gltf scene. assumes there is only one scene it the gltf file
pub fn scene(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    default_instance: bool,
    path: impl AsRef<Path>,
) -> Result<(Option<Vec<ObjectId>>, Vec<RenderId>), OneOf<(gltf::Error, String, anyhow::Error)>> {
    println!("pathbuf: {:?}", path.as_ref());
    let (document, buffers, images) = gltf::import(path).map_err(|e| OneOf::new(e))?;

    let mut render_objects: Vec<RenderId> = vec![];
    let mut game_objects: Vec<ObjectId> = vec![];
    for gltf_scene in document.scenes() {
        for node in gltf_scene.nodes() {
            let (object_id, render_ids) = load_node(
                instance,
                device,
                data,
                scene,
                &node,
                default_instance,
                &buffers,
                &images,
                None,
            )?;
            render_ids.into_iter().for_each(|r| render_objects.push(r));
            if default_instance {
                game_objects.push(object_id.unwrap())
            }
        }
    }
    if default_instance {
        Ok((Some(game_objects), render_objects))
    } else {
        Ok((None, render_objects))
    }
}

fn load_node(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    node: &Node,
    default_instance: bool,
    buffers: &Vec<buffer::Data>,
    images: &Vec<image::Data>,
    parent_transform: Option<&glam::Mat4>,
) -> Result<(Option<ObjectId>, Vec<RenderId>), OneOf<(gltf::Error, String, anyhow::Error)>> {
    let mut render_object_ids: Vec<RenderId> = vec![];
    let mut maybe_render_id: Option<RenderId> = None;
    let mut maybe_object_id: Option<ObjectId> = None;

    info!("loading node {}", node.index());
    if let Some(mesh) = node.mesh() {
        println!("mesh primitives: {:?}", mesh.primitives().count());

        for prim in mesh.primitives() {
            let mode = prim.mode();
            println!("mode: {mode:?}");

            let mut attr_map = HashMap::new();
            for attr in prim.attributes() {
                let accessor = prim.get(&attr.0).unwrap();
                let view = accessor.view().unwrap();
                println!("attribute index {:?} has type: {:?}  with data type {:?} and dimensions {:?},\
                with offset {:?}. Its data bufferview is {:?} with buffer offset {:?} and stride {:?}.",
                         accessor.index(), attr.0, accessor.data_type(), accessor.dimensions(), accessor.offset(), view.index(), view.offset(), view.stride());
                drop(view);
                let slice = get_buffer_slice(&accessor, buffers);

                attr_map.insert(attr.0, slice);
            }

            let vertices = intersperse_vertex_data(&attr_map);
            let index_acc = prim.indices().unwrap();
            let index_slice = get_buffer_slice(&index_acc, buffers);
            let indices: Vec<u32> = match index_acc.data_type() {
                gltf::accessor::DataType::U8 => {
                    println!("index data type: u8");
                    let indices = index_slice.to_vec();
                    indices.iter().map(|u| *u as u32).collect()
                }
                gltf::accessor::DataType::U16 => {
                    assert!(
                        index_slice.len() % 2 == 0,
                        "index list must be divisible by 2"
                    );
                    println!("index data type: u16");
                    let indices: &[u16] = unsafe { std::mem::transmute(index_slice) };
                    let mut u32s: Vec<u32> = vec![];
                    for i in 0..(indices.len() / 2) {
                        u32s.push(indices[i] as u32);
                    }
                    u32s
                }
                gltf::accessor::DataType::U32 => {
                    assert!(
                        index_slice.len() % 4 == 0,
                        "index list must be divisible by 4"
                    );
                    println!("index data type: u32");
                    let indices: &[u32] = unsafe { std::mem::transmute(index_slice) };
                    let mut new_indices: Vec<u32> = vec![];
                    for i in 0..indices.len() / 4 {
                        new_indices.push(indices[i]);
                    }
                    new_indices
                }
                _ => panic!("should be unsigned integer type"),
            };
            println!("indices: {:?}", indices.len());
            let pbr = prim.material().pbr_metallic_roughness();
            let texture = pbr.base_color_texture().unwrap();
            let image = &images[texture.texture().index()];

            let mut pixels = image.pixels.clone();
            match image.format {
                image::Format::R8G8B8 => pixels = interleave_alpha_channel(&pixels, 255),
                image::Format::R8G8B8A8 => (),
                format => return Err(OneOf::new(format!("weird texture format: {:?}", format))),
            }
            let vertex_data = unsafe {
                VertexData::create_vertex_data(instance, device, data, vertices, indices)
            }
            .map_err(OneOf::new)?;
            let texture_data = unsafe {
                TextureData::create_texture_from_data(
                    instance,
                    device,
                    data,
                    pixels,
                    (image.width, image.height),
                )
            }
            .map_err(OneOf::new)?;
            let render_object = unsafe {
                RenderObject::create_render_object(
                    instance,
                    device,
                    data,
                    vertex_data,
                    texture_data,
                )
            }
            .map_err(OneOf::broaden)?;
            let render_key = RenderId(scene.render_objects.insert(render_object));
            maybe_render_id = Some(render_key.clone());
            render_object_ids.push(render_key);
        }
    }
    let mut transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(parent_transform) = parent_transform {
        transform = parent_transform * transform;
    }
    let mut children = vec![];
    for child in node.children() {
        let (object_id, render_ids) = load_node(
            instance,
            device,
            data,
            scene,
            &child,
            default_instance,
            buffers,
            images,
            Some(&transform),
        )?;
        render_ids
            .into_iter()
            .for_each(|r| render_object_ids.push(r));
        if let Some(id) = object_id {
            children.push(id);
        }
    }

    if default_instance {
        transform = glam::Mat4::from_rotation_x(PI) * transform;
        let (scale, rotation, position) = transform.to_scale_rotation_translation();
        let transform = Transform {
            position,
            scale,
            rotation,
        };
        let game_object = GameObject {
            transform,
            children,
            render_object: maybe_render_id.clone(),
        };
        maybe_object_id = scene.insert_instance(game_object, maybe_render_id.clone());
    }
    Ok((maybe_object_id, render_object_ids))
}

fn get_buffer_slice<'a>(accessor: &Accessor, buffers: &'a Vec<buffer::Data>) -> &'a [u8] {
    let view = accessor.view().unwrap();
    let index = view.buffer().index();
    info!(
        "buffer index : {index:?}, length : {:?}",
        view.buffer().length()
    );

    let buffer = &buffers[index];
    let type_size = match accessor.data_type() {
        ComponentType::I8 => 1,
        ComponentType::U8 => 1,
        ComponentType::I16 => 2,
        ComponentType::U16 => 2,
        ComponentType::U32 => 4,
        ComponentType::F32 => 4,
    };
    let dimension = match accessor.dimensions() {
        Type::Scalar => 1,
        Type::Vec2 => 2,
        Type::Vec3 => 3,
        Type::Vec4 => 4,
        Type::Mat2 => 4,
        Type::Mat3 => 9,
        Type::Mat4 => 16,
    };
    &buffer.0[accessor.offset() + view.offset()
        ..accessor.offset() + view.offset() + accessor.count() * type_size * dimension]
}

fn intersperse_vertex_data(map: &HashMap<Semantic, &[u8]>) -> Vec<Vertex> {
    let positions = *map.get(&Semantic::Positions).unwrap();
    let coords = *map.get(&Semantic::TexCoords(0)).unwrap();

    assert!(
        positions.len() % 12 == 0,
        "positions must be divisible by 12"
    );
    let positions: &[Vec3] = unsafe { std::mem::transmute(positions) };

    println!("positions: {:?}", positions.len() / 12);
    assert!(coords.len() % 8 == 0, "coords must be divisible by 8");
    let coords: &[Vec2] = unsafe { std::mem::transmute(coords) };
    println!("coords: {:?}", coords.len() / 8);

    assert!(
        coords.len() / 2 == positions.len() / 3,
        "attribute lists must be of the same length, 
        but is {} and {}",
        coords.len() / 2,
        positions.len() / 3
    );
    let mut vertices = vec![];

    for i in 0..positions.len() / 12 {
        let pos = positions[i];
        let tex_coord = coords[i];
        vertices.push(Vertex {
            pos,
            color: Vec3::ONE,
            tex_coord,
        });
    }
    println!("vertex count : {:?}", vertices.len());
    vertices
}
/// turns images with type r8g8b8 to r8g8b8a8.
fn interleave_alpha_channel(rgbs: &Vec<u8>, alpha_val: u8) -> Vec<u8> {
    let a = alpha_val;
    let mut rgbas: Vec<u8> = Vec::with_capacity(rgbs.len() * (1 / 3));
    let mut count = 0;
    for b in rgbs.iter() {
        rgbas.push(b.clone());
        count += 1;
        if count == 3 {
            rgbas.push(a);
            count = 0;
        }
    }
    rgbas
}

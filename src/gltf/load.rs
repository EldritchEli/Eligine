use crate::{
    game_objects::{
        render_object::{self, RenderObject},
        transform::{self},
    },
    vulkan::{
        image_util::TextureData,
        render_app::AppData,
        vertexbuffer_util::{Vertex, VertexData},
    },
    ASSETS,
};

use bevy::render::alpha;
use core::slice;
use glam::{Vec2, Vec3, Vec4};
use gltf::{
    self,
    buffer::{self, View},
    image,
    mesh::{util::indices, Mesh},
    scene::Transform,
    Accessor, Buffer, Gltf, Material, Node, Semantic,
};
use log::info;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use terrors::OneOf;
use vulkanalia::{Device, Instance};

///loads a single gltf scene. assumes there is only one scene it the gltf file
pub fn scene(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    path: String,
) -> Result<Vec<RenderObject>, OneOf<(gltf::Error, String, anyhow::Error)>> {
    println!("pathbuf: {path:?}");
    let gltf = Gltf::open(format!("assets/{path}")).map_err(|e| OneOf::new(e))?;
    let (document, buffers, images) =
        gltf::import(format!("assets/{path}")).map_err(|e| OneOf::new(e))?;

    //let data = GltfData { nodes, meshes, accessors, buffers, views, materials };
    let mut render_objects: Vec<RenderObject> = vec![];
    for scene in document.scenes() {
        for node in scene.nodes() {
            load_node(instance, device, data, &node, &buffers, &images, None)?
                .into_iter()
                .for_each(|r| render_objects.push(r));
        }
    }
    Ok(render_objects)
}

fn load_node(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    node: &Node,
    buffers: &Vec<buffer::Data>,
    images: &Vec<image::Data>,
    parent_transform: Option<&Transform>,
) -> Result<Vec<RenderObject>, OneOf<(gltf::Error, String, anyhow::Error)>> {
    let transform = node.transform();

    let mut render_objects = vec![];

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

                let slice = get_buffer_slice(&view, buffers);
                attr_map.insert(attr.0, slice);
            }

            let vertices = intersperse_vertex_data(&attr_map);
            let index_acc = prim.indices().unwrap();
            let index_slice = get_buffer_slice(&index_acc.view().unwrap(), buffers);
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
            render_objects.push(render_object);
        }
    }

    for child in node.children() {
        load_node(
            instance,
            device,
            data,
            &child,
            buffers,
            images,
            Some(&transform),
        )?
        .into_iter()
        .for_each(|r| render_objects.push(r));
    }
    Ok(render_objects)
}

fn get_buffer_slice<'a>(view: &View, buffers: &'a Vec<buffer::Data>) -> &'a [u8] {
    let index = view.buffer().index();
    info!(
        "buffer index : {index:?}, length : {:?}",
        view.buffer().length()
    );
    let buffer = &buffers[index];
    &buffer.0[view.offset()..view.offset() + view.length()]
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
    //let positions = positions.to_vec();
    //println!("positions: {:?}", positions.len());
    assert!(coords.len() % 8 == 0, "coords must be divisible by 8");
    let coords: &[Vec2] = unsafe { std::mem::transmute(coords) };
    println!("coords: {:?}", coords.len() / 8);
    //let coords = coords.to_vec();
    //println!("coords: {:?}", coords.len());
    assert!(
        coords.len() / 2 == positions.len() / 3,
        "attribute lists must be of the same length, 
        but is {} and {}",
        coords.len(),
        positions.len()
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
/// turns images with type rgb to rgba.
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

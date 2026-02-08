use crate::asset_manager::DEFAULT_TEXTURE;
use crate::winit_app::winit_render_app::AppData;
use crate::{
    game_objects::{
        render_object::{ObjectId, PBR, RenderId, RenderObject},
        scene::{GameObject, Scene},
        transform::Transform,
    },
    vulkan::{
        image_util::TextureData,
        vertexbuffer_util::{VertexData, VertexPbr},
    },
};

use glam::{Vec2, Vec3, Vec4};
use gltf::json::accessor::{ComponentType, Type};
use gltf::{
    Accessor, Node, Semantic,
    buffer::{self},
    image,
};
use log::info;
use std::{collections::HashMap, path::Path};
use terrors::OneOf;
use vulkanalia::{Device, Instance};

///loads a single gltf scene.
pub fn scene(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    path: impl AsRef<Path>,
) -> Result<Vec<ObjectId>, OneOf<(gltf::Error, String, anyhow::Error)>> {
    let (document, buffers, images) = gltf::import(path).unwrap();
    let mut game_objects: Vec<ObjectId> = vec![];
    for gltf_scene in document.scenes() {
        for node in gltf_scene.nodes() {
            let object_id = load_node(
                instance, device, data, scene, &node, &buffers, &images, None,
            )?;

            game_objects.push(object_id.unwrap())
        }
    }

    Ok(game_objects)
}

fn load_node(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    scene: &mut Scene,
    node: &Node,
    buffers: &Vec<buffer::Data>,
    images: &Vec<image::Data>,
    parent: Option<ObjectId>,
) -> Result<Option<ObjectId>, OneOf<(gltf::Error, String, anyhow::Error)>> {
    //let mut render_object_ids: Vec<RenderId> = vec![];
    let mut render_ids: Vec<RenderId> = vec![];
    //let mut maybe_object_id: Option<ObjectId> = None;
    println!("node name: {:?}", node.name());
    if let Some(mesh) = node.mesh() {
        println!("mesh primitives: {:?}", mesh.primitives().count());

        for prim in mesh.primitives() {
            let _mode = prim.mode();
            let mut attr_map = HashMap::new();
            for attr in prim.attributes() {
                let accessor = prim.get(&attr.0).unwrap();
                let view = accessor.view().unwrap();
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
                    assert_eq!(
                        index_slice.len() % 2,
                        0,
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
                    assert_eq!(
                        index_slice.len() % 4,
                        0,
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

            let pbr = prim.material().pbr_metallic_roughness();
            let base = Vec4::from(pbr.base_color_factor());
            let (pixels, width, height) = match pbr.base_color_texture() {
                None => {
                    println!("using default texture");
                    (Vec::from(DEFAULT_TEXTURE), 1, 1)
                }
                Some(texture) => {
                    let image = &images[texture.texture().index()];
                    let mut pixels = image.pixels.clone();
                    match image.format {
                        image::Format::R8G8B8 => pixels = interleave_alpha_channel(&pixels, 255),
                        image::Format::R8G8B8A8 => (),
                        format => {
                            return Err(OneOf::new(format!("weird texture format: {:?}", format)));
                        }
                    }
                    (pixels, image.width, image.height)
                }
            };

            let vertex_data = unsafe {
                VertexData::create_vertex_data(instance, device, data, vertices, indices, false)
            }
            .map_err(OneOf::new)?;
            let texture_data = unsafe {
                TextureData::create_texture_from_data(
                    instance,
                    device,
                    data,
                    pixels,
                    (width, height),
                )
            }
            .map_err(OneOf::new)?;
            let render_object = unsafe {
                RenderObject::create_render_object(
                    instance,
                    device,
                    data,
                    vertex_data,
                    PBR { texture_data, base },
                    &mut scene.sun,
                )
            }
            .map_err(OneOf::broaden)?;
            let render_key = scene.render_objects.insert(render_object);
            render_ids.push(render_key.clone());
        }
    }
    let transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    let (scale, rotation, position) = transform.to_scale_rotation_translation();
    let transform = Transform {
        position,
        scale,
        rotation,
    };
    let game_object = GameObject {
        name: node.name().unwrap_or("unnamed").to_string(),
        transform,
        children: vec![],
        render_objects: render_ids.clone(),
        parent,
    };
    let maybe_object_id = scene.insert_instance(game_object);
    let mut children = vec![];
    for child in node.children() {
        let object_id = load_node(
            instance,
            device,
            data,
            scene,
            &child,
            buffers,
            images,
            maybe_object_id,
        )?;

        children.push(object_id.unwrap());
    }

    let game_object = scene.objects.get_mut(maybe_object_id.unwrap()).unwrap();
    game_object.children = children;
    Ok(maybe_object_id)
}

pub fn get_buffer_slice<'a>(accessor: &Accessor, buffers: &'a Vec<buffer::Data>) -> &'a [u8] {
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

pub fn intersperse_vertex_data(map: &HashMap<Semantic, &[u8]>) -> Vec<VertexPbr> {
    let positions = *map.get(&Semantic::Positions).unwrap();
    assert_eq!(positions.len() % 12, 0, "positions must be divisible by 12");
    let positions: &[Vec3] = unsafe { std::mem::transmute(positions) };

    let normals = *map.get(&Semantic::Normals).unwrap();
    let normals: &[Vec3] = unsafe { std::mem::transmute(normals) };
    let coords = map.get(&Semantic::TexCoords(0));
    let coord_flag = coords.is_some();
    let coords: &[Vec2] = if coord_flag {
        let coords = *coords.unwrap();
        assert_eq!(coords.len() % 8, 0, "coords must be divisible by 8");
        let coords: &[Vec2] = unsafe { std::mem::transmute(coords) };
        println!("coords: {:?}", coords.len() / 8);
        assert_eq!(
            coords.len() / 2,
            positions.len() / 3,
            "attribute lists must be of the same length,
        but is {} and {}",
            coords.len() / 2,
            positions.len() / 3
        );
        coords
    } else {
        &[]
    };

    let mut vertices = vec![];

    for i in 0..positions.len() / 12 {
        let pos = positions[i];
        let tex_coord = if coord_flag { coords[i] } else { Vec2::ZERO };
        let normal = normals[i];
        vertices.push(VertexPbr {
            pos,
            normal,
            tex_coord,
        });
    }
    println!("vertex count : {:?}", vertices.len());
    vertices
}
/// turns images with type r8g8b8 to r8g8b8a8.
pub fn interleave_alpha_channel(rgbs: &Vec<u8>, alpha_val: u8) -> Vec<u8> {
    let a = alpha_val;
    let mut rgbas: Vec<u8> = Vec::with_capacity(rgbs.len() * (1 / 3) as usize);
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

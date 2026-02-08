/*use crate::asset_manager::DEFAULT_TEXTURE;
use crate::asset_manager::load::{
    get_buffer_slice, interleave_alpha_channel, intersperse_vertex_data,
};
use crate::game_objects::scene::Sun;
use crate::winit_app::winit_render_app::AppData;
use crate::{
    game_objects::{
        render_object::{ObjectId, PBR, RenderId, RenderObject},
        scene::GameObject,
    },
    vulkan::{image_util::TextureData, vertexbuffer_util::VertexData},
};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Commands;
use bevy::math::{Mat4, Vec4};
use bevy::prelude::Transform;
use gltf::{Node, buffer, image};
use std::{collections::HashMap, path::Path};
use terrors::OneOf;
use tracing::error;
use vulkanalia::{Device, Instance};

pub fn scene(
    mut command: Commands,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    path: impl AsRef<Path>,
    sun: &Sun,
) {
    let (document, buffers, images) = gltf::import(path).unwrap();
    for gltf_scene in document.scenes() {
        for node in gltf_scene.nodes() {
            let Ok(object_id) = load_pbr(
                &mut command,
                instance,
                device,
                data,
                &node,
                &buffers,
                &images,
                None,
                &sun,
            ) else {
                error!("failed to load pbr node");
                return;
            };
        }
    }
}
//instanciates a number of render_sources and their instances, assumes that they are compatible with normal pbr material
pub fn load_pbr(
    commands: &mut Commands,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    node: &Node,
    buffers: &Vec<buffer::Data>,
    images: &Vec<image::Data>,
    parent: Option<ObjectId>,
    sun: &Sun,
) -> Result<Option<Entity>, OneOf<(gltf::Error, String, anyhow::Error)>> {
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
                let slice = get_buffer_slice(&accessor, buffers);
                attr_map.insert(attr.0, slice);
            }

            let vertices = intersperse_vertex_data(&attr_map);
            let indices = format_indices(buffers, &prim);

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
                    &sun,
                )
            }
            .map_err(OneOf::broaden)?;
            let render_key = scene.render_objects.insert(render_object);
            render_ids.push(render_key.clone());
        }
    }
    let transform = Mat4::from_cols_array_2d(&node.transform().matrix());

    let (scale, rotation, translation) = transform.to_scale_rotation_translation();
    let transform = Transform {
        translation,
        rotation,
        scale,
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
        let object_id = load_pbr(
            commands,
            instance,
            device,
            data,
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

fn format_indices(buffers: &Vec<buffer::Data>, prim: &gltf::Primitive<'_>) -> Vec<u32> {
    let index_acc = prim.indices().unwrap();
    let index_slice = get_buffer_slice(&index_acc, buffers);
    let indices: Vec<u32> = match index_acc.data_type() {
        gltf::accessor::DataType::U8 => {
            let indices = index_slice.to_vec();
            indices.iter().map(|u| *u as u32).collect()
        }
        gltf::accessor::DataType::U16 => {
            assert_eq!(
                index_slice.len() % 2,
                0,
                "index list must be divisible by 2"
            );
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
            let indices: &[u32] = unsafe { std::mem::transmute(index_slice) };
            let mut new_indices: Vec<u32> = vec![];
            for i in 0..indices.len() / 4 {
                new_indices.push(indices[i]);
            }
            new_indices
        }
        _ => panic!("should be unsigned integer type"),
    };
    indices
}*/

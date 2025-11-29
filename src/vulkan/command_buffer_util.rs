#![allow(unsafe_op_in_unsafe_fn)]
use crate::game_objects::scene::Scene;
use crate::gui::gui::Gui;
use crate::vulkan::render_app::AppData;
use bevy::picking::window;
use egui::Rect;
use iced::widget::shader::wgpu::vertex_attr_array;
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{Device, vk};
use winit::window::Window;

pub unsafe fn create_command_buffers(
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    window: &Window,
    mut gui: Option<&Gui>,
) -> anyhow::Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
            .inheritance_info(&inheritance); // Optional.

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);

        //device.cmd_draw(*command_buffer, VERTICES.len() as u32, 1, 0, 0);
        if let Some(skybox) = &scene.skybox {
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.gui_pipeline,
            );
            let descriptorset = &gui
                .as_ref()
                .unwrap()
                .vertex_data
                .iter()
                .next()
                .unwrap()
                .descriptor_sets;
            device.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.gui_pipeline_layout,
                0,
                &[descriptorset[i]],
                &[],
            );
            //device.cmd_draw(*command_buffer, 4, 1, 0, 0);

            let mut vertex_data = gui.as_mut().unwrap().vertex_data.clone();
            vertex_data.reverse();
            for object in vertex_data {
                device.cmd_bind_vertex_buffers(
                    *command_buffer,
                    0,
                    &[object.vertex_data.vertex_buffer],
                    &[0],
                );
                device.cmd_bind_index_buffer(
                    *command_buffer,
                    object.vertex_data.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    data.gui_pipeline_layout,
                    0,
                    &[object.descriptor_sets[i]],
                    &[],
                );
                println!("Rect:  {:?}", object.rect);
                let Rect { mut min, mut max } = object.rect;
                let scale = window.scale_factor() as f32;
                min.x *= scale;
                min.y *= scale;
                max.x *= scale;
                max.y *= scale;
                device.cmd_set_scissor(
                    *command_buffer,
                    0,
                    &[vk::Rect2D::builder()
                        .offset(
                            vk::Offset2D::builder()
                                .x(min.x.round() as i32)
                                .y(min.y.round() as i32)
                                .build(),
                        )
                        .extent(
                            vk::Extent2D::builder()
                                .width((max.x.round() - min.x) as u32)
                                .height((max.y.round() - min.y) as u32)
                                .build(),
                        )
                        .build()],
                );

                device.cmd_draw_indexed(
                    *command_buffer,
                    object.vertex_data.indices.len() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            }
        }

        device.cmd_next_subpass(*command_buffer, vk::SubpassContents::INLINE);
        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pbr_pipeline,
        );

        for (_, object) in scene.render_objects.iter() {
            device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &[object.vertex_data.vertex_buffer],
                &[0],
            );
            device.cmd_bind_index_buffer(
                *command_buffer,
                object.vertex_data.index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            device.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.pbr_pipeline_layout,
                0,
                &[object.descriptor_sets[i]],
                &[],
            );

            let f32_push_data = (scene.camera.transform.matrix().inverse()).to_cols_array();
            let push_data: [u8; 64] = std::mem::transmute(f32_push_data);
            device.cmd_push_constants(
                *command_buffer,
                data.pbr_pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &push_data,
            );

            device.cmd_draw_indexed(
                *command_buffer,
                object.vertex_data.indices.len()/*INDICES.len()*/ as u32,
                object.instances.len() as u32,
                0,
                0,
                0,
            );
        }
        device.cmd_next_subpass(*command_buffer, vk::SubpassContents::INLINE);
        /*if let Some(gui) = gui {
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.gui_pipeline,
            );
            for v in &gui.vertex_data {
                /*  device.cmd_bind_vertex_buffers(
                    *command_buffer,
                    0,
                    &[v.vertex_data.vertex_buffer],
                    &[0],
                );
                device.cmd_bind_index_buffer(
                    *command_buffer,
                    v.vertex_data.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );*/
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    data.gui_pipeline_layout,
                    0,
                    &[v.descriptor_sets[i]],
                    &[],
                );
                println!("vertices for gui: {:?}", v.vertex_data.vertices.len());
                //v.vertex_data.vertices.iter().for_each(|i| {
                //    println!("vert: {:?}", i);
                //});
                //device.cmd_draw(*command_buffer, 48, 1, 0, 0);
            }
            device.cmd_draw(*command_buffer, 4, 1, 0, 0);
        }*/

        device.cmd_end_render_pass(*command_buffer);
        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}
/*
pub unsafe fn mock_command_buffers(
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    gui: &Option<Gui>,
) -> anyhow::Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
            .inheritance_info(&inheritance); // Optional.

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);

        //device.cmd_draw(*command_buffer, VERTICES.len() as u32, 1, 0, 0);
        if let Some(skybox) = &scene.skybox {
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.skybox_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.skybox_pipeline_layout,
                0,
                &[skybox.descriptors[i]],
                &[],
            );
            //device.cmd_draw(*command_buffer, 4, 1, 0, 0);
        }

        device.cmd_next_subpass(*command_buffer, vk::SubpassContents::INLINE);
        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pbr_pipeline,
        );

        for (_, object) in scene.render_objects.iter() {
            device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &[object.vertex_data.vertex_buffer],
                &[0],
            );
            device.cmd_bind_index_buffer(
                *command_buffer,
                object.vertex_data.index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            device.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.pbr_pipeline_layout,
                0,
                &[object.descriptor_sets[i]],
                &[],
            );

            let f32_push_data = (scene.camera.transform.matrix().inverse()).to_cols_array();
            let push_data: [u8; 64] = std::mem::transmute(f32_push_data);
            device.cmd_push_constants(
                *command_buffer,
                data.pbr_pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &push_data,
            );

            /*device.cmd_draw_indexed(
                *command_buffer,
                object.vertex_data.indices.len()/*INDICES.len()*/ as u32,
                object.instances.len() as u32,
                0,
                0,
                0,
            );*/
        }
        device.cmd_next_subpass(*command_buffer, vk::SubpassContents::INLINE);
        if let Some(gui) = gui {
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                gui.pipeline,
            );
            for v in &gui.vertex_data {
                /*  device.cmd_bind_vertex_buffers(
                    *command_buffer,
                    0,
                    &[v.vertex_data.vertex_buffer],
                    &[0],
                );
                device.cmd_bind_index_buffer(
                    *command_buffer,
                    v.vertex_data.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );*/
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    gui.pipeline_layout,
                    0,
                    &[v.descriptor_sets[i]],
                    &[],
                );
                println!("vertices for gui: {:?}", v.vertex_data.vertices.len());
                //v.vertex_data.vertices.iter().for_each(|i| {
                //    println!("vert: {:?}", i);
                //});
                //device.cmd_draw(*command_buffer, 48, 1, 0, 0);
            }
            device.cmd_draw(*command_buffer, 4, 1, 0, 0);
        }

        device.cmd_end_render_pass(*command_buffer);
        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}*/

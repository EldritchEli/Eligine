#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use crate::game_objects::scene::Scene;
use crate::gui::gui::Gui;
use crate::winit_app::winit_render_app::AppData;
use egui::Rect;
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{Device, vk};
use winit::window::Window;

pub unsafe fn create_command_buffers(
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    window: &Window,
    gui: Option<&Gui>,
) -> anyhow::Result<()> {
    for i in 0..data.framebuffers.len() {
        let command_center = &mut data.command_centers[i];
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_center.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        command_center.command_buffers = device.allocate_command_buffers(&allocate_info)?;
        assert_eq!(command_center.command_buffers.len(), 1);
        create_command_buffer(device, scene, data, window, gui, i)?;
    }
    Ok(())
}

pub unsafe fn create_command_buffer(
    device: &Device,
    scene: &mut Scene,
    data: &mut AppData,
    window: &Window,
    gui: Option<&Gui>,
    i: usize,
) -> anyhow::Result<()> {
    let command_buffer = &data.command_centers[i].command_buffers[0];
    device.reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())?;
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
    if let Some(gui) = &gui
        && gui.enabled
        && !gui.render_objects.is_empty()
    {
        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.gui_pipeline,
        );

        device.cmd_push_constants(
            *command_buffer,
            data.pbr_pipeline_layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            0,
            &data.pbr_push_contant.data(),
        );
        for object in &gui.render_objects[i] {
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
                &[object.descriptor_set],
                &[],
            );
            let Rect { min, max } = object.rect;
            let ppp = gui.egui_state.egui_ctx().pixels_per_point();
            let clip_x = ppp * min.x;
            let clip_y = ppp * min.y;
            let clip_w = max.x * ppp - clip_x;
            let clip_h = max.y * ppp - clip_y;

            let scissors = [vk::Rect2D::builder()
                .offset(
                    vk::Offset2D::builder()
                        .x((clip_x as i32).max(0))
                        .y((clip_y as i32).max(0))
                        .build(),
                )
                .extent(
                    vk::Extent2D::builder()
                        .width((clip_w as u32).min(data.swapchain_extent.width))
                        .height((clip_h as u32).min(data.swapchain_extent.height))
                        .build(),
                )
                .build()];
            device.cmd_set_scissor(*command_buffer, 0, &scissors);

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
    let viewports = if let Some(gui) = gui
        && gui.enabled
    {
        let Rect { min, max } = gui.callback;

        let ppp = gui.egui_state.egui_ctx().pixels_per_point();
        let x = ppp * min.x;
        let y = ppp * min.y;
        let w = max.x * ppp - x;
        let h = max.y * ppp - y;
        &[vk::Viewport::builder()
            .x(x)
            .y(y)
            .width(w)
            .height(h)
            .min_depth(0.0)
            .max_depth(1.0)]
    } else {
        &[vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(data.swapchain_extent.width as f32)
            .height(data.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)]
    };
    device.cmd_set_viewport(*command_buffer, 0, viewports);
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

        device.cmd_push_constants(
            *command_buffer,
            data.pbr_pipeline_layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            0,
            &data.pbr_push_contant.data(),
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
    if let Some(skybox) = &scene.skybox
        && !skybox.descriptor_sets.is_empty()
    {
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
            &[skybox.descriptor_sets[i]],
            &[],
        );
        device.cmd_draw(*command_buffer, 4, 1, 0, 0);
    }

    device.cmd_end_render_pass(*command_buffer);
    device.end_command_buffer(*command_buffer)?;

    Ok(())
}

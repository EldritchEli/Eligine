#![allow(unsafe_op_in_unsafe_fn)]
use crate::vulkan::shader_module_util::create_shader_module;
use crate::vulkan::uniform_buffer_object::PbrPushConstant;
use crate::vulkan::vertexbuffer_util::{Vertex, VertexGui, VertexPbr};
use crate::vulkan::winit_render_app::AppData;
use vulkanalia::vk::{DeviceV1_0, Handle, HasBuilder};
use vulkanalia::{Device, vk};

pub unsafe fn create_pbr_pipeline(
    device: &Device,
    data: &mut AppData,
    subpass_order: u32,
) -> std::result::Result<(), anyhow::Error> {
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .build();
    let vert = std::fs::read("src/shaders/spv/pbr_vert.spv").unwrap();
    let frag = std::fs::read("src/shaders/spv/pbr_frag.spv").unwrap();

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    let push_range = vk::PushConstantRange::builder()
        .offset(0)
        .size(size_of::<PbrPushConstant>() as u32)
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .build();
    //specialization_info for shader constants!!
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[<VertexPbr>::binding_description()];
    let attribute_descriptions = <VertexPbr>::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(data.swapchain_extent);

    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::empty())
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        // Enable sample shading in the pipeline.
        .sample_shading_enable(true)
        // Minimum fraction for sample shading; closer to one is smoother.
        .min_sample_shading(0.2)
        .rasterization_samples(data.msaa_samples);

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.0) // Optional.
        .max_depth_bounds(1.0) // Optional.
        .stencil_test_enable(false);
    //.front(/* vk::StencilOpState */) // Optional.
    //  .back(/* vk::StencilOpState */); // Optional.

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

    let set_layouts = &[data.pbr_descriptor_set_layout];
    let push_ranges = [push_range];
    let mut layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(&push_ranges);
    layout_info.push_constant_range_count = 1;
    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(&dynamic_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(data.render_pass)
        .subpass(subpass_order);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
    data.pbr_pipeline_layout = pipeline_layout;
    data.pbr_pipeline = pipeline;
    Ok(())
}

pub unsafe fn skybox_pipeline(
    device: &Device,
    data: &mut AppData,
    subpass_position: u32,
) -> anyhow::Result<()> {
    let vert = std::fs::read("src/shaders/spv/skybox_vert.spv").unwrap();
    let frag = std::fs::read("src/shaders/spv/skybox_frag.spv").unwrap();

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    //specialization_info for shader constants!!
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    //let binding_descriptions = &[SimpleVertex::binding_description()];
    //let attribute_descriptions = SimpleVertex::attribute_descriptions();
    let binding_descriptions = &[VertexGui::binding_description()];
    let attribute_descriptions = VertexGui::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(data.swapchain_extent);

    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::empty())
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        // Enable sample shading in the pipeline.
        .sample_shading_enable(true)
        // Minimum fraction for sample shading; closer to one is smoother.
        .min_sample_shading(0.2)
        .rasterization_samples(data.msaa_samples);

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.0) // Optional.
        .max_depth_bounds(1.0) // Optional.
        .stencil_test_enable(false);
    //.front(/* vk::StencilOpState */) // Optional.
    //  .back(/* vk::StencilOpState */); // Optional.

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

    let set_layouts = &[data.skybox_descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);
    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
        .primitive_restart_enable(true)
        .build();
    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .dynamic_state(&dynamic_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(data.render_pass)
        .subpass(subpass_position);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    data.skybox_pipeline_layout = pipeline_layout;
    data.skybox_pipeline = pipeline;
    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
    Ok(())
}

pub unsafe fn gui_pipeline(
    device: &Device,
    data: &mut AppData,
    subpass_position: u32,
) -> anyhow::Result<()> {
    let vert = std::fs::read("src/shaders/spv/gui_vert.spv").unwrap();
    let frag = std::fs::read("src/shaders/spv/gui_frag.spv").unwrap();
    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    //specialization_info for shader constants!!
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[VertexGui::binding_description()];
    let attribute_descriptions = VertexGui::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    //let scissor = vk::Rect2D::builder()
    //  .offset(vk::Offset2D { x: 0, y: 0 })
    //    .extent(data.swapchain_extent);

    let viewports = &[viewport];
    //let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissor_count(1);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .line_width(1.0)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state =
        vk::PipelineMultisampleStateCreateInfo::builder().rasterization_samples(data.msaa_samples);
    let stencil_op = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .build();

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.0) // Optional.
        .max_depth_bounds(1.0) // Optional.
        .stencil_test_enable(false)
        .front(stencil_op) // Optional.
        .back(stencil_op); // Optional.

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .build()];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&color_blend_attachments)
        .build();

    let dynamic_states = &[
        // vk::DynamicState::VIEWPORT,
        //vk::DynamicState::LINE_WIDTH,
        vk::DynamicState::SCISSOR,
    ];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

    let set_layouts = &[data.gui_descriptor_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);
    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();
    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .dynamic_state(&dynamic_state)
        .render_pass(data.render_pass)
        .subpass(subpass_position);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    data.gui_pipeline_layout = pipeline_layout;
    data.gui_pipeline = pipeline;
    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
    Ok(())
}

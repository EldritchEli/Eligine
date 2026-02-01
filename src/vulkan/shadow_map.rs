#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use vulkanalia::{
    Device, Instance,
    vk::{self, DeviceV1_0, Handle, HasBuilder, ImageView},
};

use crate::vulkan::{
    framebuffer_util::get_depth_format,
    image_util::{TextureData, create_image, create_image_view},
    shader_module_util::create_shader_module,
    winit_render_app::AppData,
};

#[derive(Clone, Debug)]
pub struct ShadowMap {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub framebuffer: vk::Framebuffer,
}

impl ShadowMap {
    pub unsafe fn new(
        instance: &Instance,
        data: &AppData,
        device: &Device,
    ) -> anyhow::Result<Self> {
        // Image + Image Memory

        let format = get_depth_format(instance, data)?;

        let (depth_image, depth_image_memory) = create_image(
            instance,
            device,
            data,
            data.swapchain_extent.width,
            data.swapchain_extent.height,
            1,
            1,
            data.msaa_samples,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Image View

        let depth_image_view = create_image_view(
            device,
            data.depth_image,
            format,
            vk::ImageAspectFlags::DEPTH,
            1,
            1,
        )?;
        let sampler = TextureData::create_texture_sampler(1, device)?;
        let descriptor_set_layout = descriptor_set_layout(device)?;
        let descriptor_set = descriptor_set(
            descriptor_set_layout,
            data.descriptor_pool,
            device,
            depth_image_view,
            sampler,
        )?;
        let attachments = [depth_image_view];
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(data.render_pass)
            .attachments(&attachments)
            .width(data.swapchain_extent.width)
            .height(data.swapchain_extent.height)
            .layers(1);

        let framebuffer = device.create_framebuffer(&create_info, None)?;
        Ok(ShadowMap {
            image: depth_image,
            image_memory: depth_image_memory,
            image_view: depth_image_view,
            sampler,
            descriptor_set_layout,
            descriptor_set,
            framebuffer,
        })
    }
}
// To be used by other shaders
pub unsafe fn descriptor_set_layout(device: &Device) -> anyhow::Result<vk::DescriptorSetLayout> {
    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    Ok(device.create_descriptor_set_layout(&info, None)?)
}
pub unsafe fn descriptor_set(
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    device: &Device,
    image_view: ImageView,
    sampler: vk::Sampler,
) -> anyhow::Result<vk::DescriptorSet> {
    let layouts = vec![layout];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptor_sets = device.allocate_descriptor_sets(&info)?;
    let info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(image_view)
        .sampler(sampler);

    let image_info = &[info];
    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(descriptor_sets[0])
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_info);

    unsafe { device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]) };
    Ok(descriptor_sets[0])
}

pub unsafe fn shadow_map_pipeline(
    device: &Device,
    data: &mut AppData,
    subpass_order: u32,
) -> std::result::Result<(), anyhow::Error> {
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .build();
    let vert = std::fs::read("src/shaders/spv/shadow.spv").unwrap();
    let frag = std::fs::read("src/shaders/spv/shadow.spv").unwrap();

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

    let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];

    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(dynamic_states);

    let set_layouts = &[data.pbr_descriptor_set_layout];
    let mut layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);
    layout_info.push_constant_range_count = 1;
    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(&dynamic_state)
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

pub unsafe fn shadow_map_render_pass(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {
    // Attachments

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(unsafe { get_depth_format(instance, data) }?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    // Subpasses

    let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let subpass_description = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .depth_stencil_attachment(&depth_stencil_attachment_ref);
    // Dependencies

    // Create

    let attachments = &[depth_stencil_attachment];
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL) // main
        .dst_subpass(0) //skybox
        .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
        .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

    let subpasses = &[subpass_description];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    data.render_pass = unsafe { device.create_render_pass(&info, None) }?;

    Ok(())
}

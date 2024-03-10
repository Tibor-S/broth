use vulkanalia::{
    bytecode::Bytecode,
    vk::{self, DeviceV1_0, ErrorCode, Handle, HasBuilder},
    Device,
};

use crate::vertex::{Vertex2, Vertex3};

// TODO: Look into creating an interface specifying wether
// TODO: the pipeline is 2D or 3D. Will use two different
// TODO: shaders and vertex structs.
pub unsafe fn create_pipeline(
    device: &Device,
    pipeline: &mut vk::Pipeline,
    pipeline_layout: &mut vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
) -> Result<()> {
    let vert = include_bytes!("../shaders/vert.spv");
    let frag = include_bytes!("../shaders/frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex3::binding_description()];
    let attribute_descriptions = Vertex3::attribute_descriptions();
    let vertex_input_state =
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state =
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    // TODO: Wtf is a scissor?
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state =
        vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);

    let rasterization_state =
        vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

    let multisample_state =
        vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .rasterization_samples(msaa_samples);

    let depth_stencil_state =
        vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0) // Optional.
            .max_depth_bounds(1.0) // Optional.
            .stencil_test_enable(false);
    // .front(/* vk::StencilOpState */) // Optional.
    // .back(/* vk::StencilOpState */); // Optional.

    // * More parameters vk::BlendFactor vk::BlendOp + documentation
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
    let color_blend_state =
        vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let set_layouts = &[descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts);
    *pipeline_layout =
        device.create_pipeline_layout(&layout_info, None)?;

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
        .layout(*pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null()) // Optional.
        .base_pipeline_index(-1); // Optional.

    *pipeline = device
        .create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[info],
            None,
        )?
        .0
        .get(0)
        .unwrap()
        .to_owned();

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(())
}
pub unsafe fn create_pipeline_2d(
    device: &Device,
    pipeline: &mut vk::Pipeline,
    pipeline_layout: &mut vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
) -> Result<()> {
    let vert = include_bytes!("../shaders/2d_vert.spv");
    let frag = include_bytes!("../shaders/frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex2::binding_description()];
    let attribute_descriptions = Vertex2::attribute_descriptions();
    let vertex_input_state =
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state =
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    // TODO: Wtf is a scissor?
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state =
        vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);

    let rasterization_state =
        vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

    let multisample_state =
        vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .rasterization_samples(msaa_samples);

    // let depth_stencil_state =
    //     vk::PipelineDepthStencilStateCreateInfo::builder()
    //         .depth_test_enable(true)
    //         .depth_write_enable(true)
    //         .depth_compare_op(vk::CompareOp::LESS)
    //         .depth_bounds_test_enable(false)
    //         .min_depth_bounds(0.0) // Optional.
    //         .max_depth_bounds(1.0) // Optional.
    //         .stencil_test_enable(false);
    // .front(/* vk::StencilOpState */) // Optional.
    // .back(/* vk::StencilOpState */); // Optional.

    // * More parameters vk::BlendFactor vk::BlendOp + documentation
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
    let color_blend_state =
        vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let set_layouts = &[descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts);
    *pipeline_layout =
        device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(*pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null()) // Optional.
        .base_pipeline_index(-1); // Optional.

    *pipeline = device
        .create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[info],
            None,
        )?
        .0
        .get(0)
        .unwrap()
        .to_owned();

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(())
}

pub unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> Result<vk::ShaderModule> {
    let bytecode = match Bytecode::new(bytecode) {
        Ok(b) => b,
        Err(e) => {
            panic!("Failed to create shader module, make sure provided shader code is valid. Returned error: {}", e)
        }
    };
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PipelineError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
}
type Result<T> = std::result::Result<T, PipelineError>;

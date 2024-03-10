use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder, InstanceV1_0},
    Device, Instance,
};

use crate::{
    image::{create_image, ImageError},
    image_view::{create_image_view, ImageViewError},
};

pub unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
    render_pass: &mut vk::RenderPass,
) -> Result<()> {
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment =
        vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_resolve_attachment_ref =
        vk::AttachmentReference::builder()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let resolve_attachments = &[color_resolve_attachment_ref];

    let depth_stencil_attachment =
        vk::AttachmentDescription::builder()
            .format(get_depth_format(instance, physical_device)?)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            );
    let depth_stencil_attachment_ref =
        vk::AttachmentReference::builder().attachment(1).layout(
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_ref)
        .resolve_attachments(resolve_attachments);

    let attachments = &[
        color_attachment,
        depth_stencil_attachment,
        color_resolve_attachment,
    ];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    *render_pass = device.create_render_pass(&info, None)?;

    Ok(())
}

pub unsafe fn get_depth_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        physical_device,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

pub unsafe fn create_render_pass_2d(
    instance: &Instance,
    device: &Device,
    swapchain_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
    render_pass: &mut vk::RenderPass,
) -> Result<()> {
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment =
        vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_resolve_attachment_ref =
        vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let resolve_attachments = &[color_resolve_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .resolve_attachments(resolve_attachments);

    let attachments = &[color_attachment, color_resolve_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    *render_pass = device.create_render_pass(&info, None)?;

    Ok(())
}

pub unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance
                .get_physical_device_format_properties(
                    physical_device,
                    *f,
                );
            match tiling {
                vk::ImageTiling::LINEAR => properties
                    .linear_tiling_features
                    .contains(features),
                vk::ImageTiling::OPTIMAL => properties
                    .optimal_tiling_features
                    .contains(features),
                _ => false,
            }
        })
        .ok_or(RenderPassError::SupportError)
}

pub unsafe fn create_depth_objects(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    depth_image: &mut vk::Image,
    depth_image_memory: &mut vk::DeviceMemory,
    depth_image_view: &mut vk::ImageView,
) -> Result<()> {
    let format = get_depth_format(instance, physical_device)?;
    (*depth_image, *depth_image_memory) = create_image(
        instance,
        device,
        physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // Image View

    *depth_image_view = create_image_view(
        device,
        *depth_image,
        format,
        vk::ImageAspectFlags::DEPTH,
        1,
    )?;

    Ok(())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RenderPassError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    ImageViewError(#[from] ImageViewError),
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error("Failed to find supported format.")]
    SupportError,
}
type Result<T> = std::result::Result<T, RenderPassError>;

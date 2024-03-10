use vulkanalia::{
    vk::{
        self, DeviceV1_0, ErrorCode, Handle, HasBuilder, ImageView,
        KhrSurfaceExtension, KhrSwapchainExtension,
    },
    Device, Instance, VkResult,
};
use winit::window::Window;

use crate::{
    image_view::{create_image_view, ImageViewError},
    queue::{QueueError, QueueFamilyIndices},
    MAX_FRAMES_IN_FLIGHT,
};

pub unsafe fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    swapchain: &mut vk::SwapchainKHR,
    swapchain_images: &mut Vec<vk::Image>,
    swapchain_format: &mut vk::Format,
    swapchain_extent: &mut vk::Extent2D,
) -> Result<()> {
    let indices =
        QueueFamilyIndices::get(instance, surface, physical_device)?;
    let support =
        SwapchainSupport::get(instance, surface, physical_device)?;

    let surface_format =
        get_swapchain_surface_format(&support.formats);
    let present_mode =
        get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);
    let mut image_count = support.capabilities.min_image_count + 1;

    if support.capabilities.max_image_count != 0
        && image_count > support.capabilities.max_image_count
    {
        image_count = support.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    *swapchain = device.create_swapchain_khr(&info, None)?;
    *swapchain_images =
        device.get_swapchain_images_khr(*swapchain)?;
    *swapchain_format = surface_format.format;
    *swapchain_extent = extent;
    Ok(())
}

pub unsafe fn create_swapchain_image_views(
    device: &Device,
    swapchain_images: &[vk::Image],
    swapchain_format: vk::Format,
    swapchain_image_views: &mut Vec<ImageView>,
) -> Result<()> {
    *swapchain_image_views = swapchain_images
        .iter()
        .map(|i| {
            create_image_view(
                device,
                *i,
                swapchain_format,
                vk::ImageAspectFlags::COLOR,
                1,
            )
            .map_err(|e| e.into())
        })
        .collect::<Result<Vec<ImageView>>>()?;

    Ok(())
}

pub fn get_swapchain_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

pub fn get_swapchain_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let size = window.inner_size();
        let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
        vk::Extent2D::builder()
            .width(clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
                size.width,
            ))
            .height(clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
                size.height,
            ))
            .build()
    }
}

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub unsafe fn get(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        Ok(Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(
                    physical_device,
                    surface,
                )?,
            formats: instance
                .get_physical_device_surface_formats_khr(
                    physical_device,
                    surface,
                )?,
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(
                    physical_device,
                    surface,
                )?,
        })
    }
}

pub unsafe fn create_framebuffers(
    device: &Device,
    swapchain_image_views: &[vk::ImageView],
    color_image_view: vk::ImageView,
    depth_image_view: vk::ImageView,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    framebuffers: &mut Vec<vk::Framebuffer>,
) -> Result<()> {
    *framebuffers = swapchain_image_views
        .iter()
        .map(|i| {
            let attachments =
                &[color_image_view, depth_image_view, *i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<VkResult<Vec<_>>>()?;

    Ok(())
}

pub unsafe fn create_framebuffers_2d(
    device: &Device,
    swapchain_image_views: &[vk::ImageView],
    color_image_view: vk::ImageView,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    framebuffers: &mut Vec<vk::Framebuffer>,
) -> Result<()> {
    *framebuffers = swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[color_image_view, *i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<VkResult<Vec<_>>>()?;

    Ok(())
}

pub unsafe fn create_sync_objects(
    device: &Device,
    swapchain_images: &[vk::Image],
    image_available_semaphores: &mut Vec<vk::Semaphore>,
    render_finished_semaphores: &mut Vec<vk::Semaphore>,
    in_flight_fences: &mut Vec<vk::Fence>,
    images_in_flight: &mut Vec<vk::Fence>,
) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    *images_in_flight =
        swapchain_images.iter().map(|_| vk::Fence::null()).collect();

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        in_flight_fences
            .push(device.create_fence(&fence_info, None)?);
    }

    Ok(())
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum SwapchainError {
    #[error(transparent)]
    QueueError(#[from] QueueError),
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    ImageViewError(#[from] ImageViewError),
}
type Result<T> = std::result::Result<T, SwapchainError>;

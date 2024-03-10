use vulkanalia::{
    vk::{self, ErrorCode},
    Device, Instance,
};

use crate::{
    image::{create_image, ImageError},
    image_view::{create_image_view, ImageViewError},
};

pub unsafe fn create_color_objects(
    instance: &Instance,
    device: &Device,
    color_image: &mut vk::Image,
    color_image_memory: &mut vk::DeviceMemory,
    color_image_view: &mut vk::ImageView,
    physical_device: vk::PhysicalDevice,
    swapchain_extent: vk::Extent2D,
    swapchain_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
) -> Result<()> {
    (*color_image, *color_image_memory) = create_image(
        instance,
        device,
        physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        swapchain_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    *color_image_view = create_image_view(
        device,
        *color_image,
        swapchain_format,
        vk::ImageAspectFlags::COLOR,
        1,
    )?;

    Ok(())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ColorError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error(transparent)]
    ImageViewError(#[from] ImageViewError),
}
type Result<T> = std::result::Result<T, ColorError>;

use vulkanalia::{
    vk::{self, ErrorCode},
    Device, Instance,
};

use crate::{
    app::AppData,
    image::{create_image, ImageError},
    image_view::{create_image_view, ImageViewError},
};

pub unsafe fn create_color_objects(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let (color_image, color_image_memory) = create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        data.msaa_samples,
        data.swapchain_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.color_image = color_image;
    data.color_image_memory = color_image_memory;

    data.color_image_view = create_image_view(
        device,
        data.color_image,
        data.swapchain_format,
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

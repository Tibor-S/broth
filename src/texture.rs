use std::{fs::File, ptr::copy_nonoverlapping as memcpy};

use png::DecodingError;
use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder},
    Device, Instance,
};

use crate::{
    buffer::{create_buffer, BufferError},
    image::{
        copy_buffer_to_image, create_image, generate_mipmaps,
        transition_image_layout, ImageError,
    },
    image_view::{create_image_view, ImageViewError},
};

pub unsafe fn create_texture_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    mip_levels: &mut u32,
    texture_image: &mut vk::Image,
    texture_image_memory: &mut vk::DeviceMemory,
) -> Result<()> {
    let image =
        File::open("resources/viking_room.png").map_err(|e| {
            TextureError::FileOpenError(
                "resources/viking_room.png".into(),
                e.to_string(),
            )
        })?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();
    *mip_levels =
        (width.max(height) as f32).log2().floor() as u32 + 1;

    if width != 1024
        || height != 1024
        || reader.info().color_type != png::ColorType::Rgba
    {
        return Err(TextureError::UnsupportedTextureError);
    }

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT
            | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

    device.unmap_memory(staging_buffer_memory);

    (*texture_image, *texture_image_memory) = create_image(
        instance,
        device,
        physical_device,
        width,
        height,
        *mip_levels,
        vk::SampleCountFlags::_1,
        // ! SRGB is not necessarily supported
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    transition_image_layout(
        device,
        command_pool,
        graphics_queue,
        *texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        *mip_levels,
    )?;

    copy_buffer_to_image(
        device,
        command_pool,
        graphics_queue,
        staging_buffer,
        *texture_image,
        width,
        height,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    generate_mipmaps(
        instance,
        device,
        physical_device,
        command_pool,
        graphics_queue,
        *texture_image,
        vk::Format::R8G8B8A8_SRGB,
        width,
        height,
        *mip_levels,
    )?;

    Ok(())
}

pub unsafe fn create_texture_image_view(
    device: &Device,
    texture_image: &vk::Image,
    mip_levels: &u32,
    texture_image_view: &mut vk::ImageView,
) -> Result<()> {
    *texture_image_view = create_image_view(
        device,
        *texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
        *mip_levels,
    )?;

    Ok(())
}

pub unsafe fn create_texture_sampler(
    device: &Device,
    mip_levels: &u32,
    texture_sampler: &mut vk::Sampler,
) -> Result<()> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.0) // Optional.
        .max_lod(*mip_levels as f32)
        .mip_lod_bias(0.0); // Optional.

    *texture_sampler = device.create_sampler(&info, None)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum TextureError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    DecodingError(#[from] DecodingError),
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error(transparent)]
    ImageViewError(#[from] ImageViewError),
    #[error(transparent)]
    BufferError(#[from] BufferError),

    #[error("Failed to open texture image {0} with error: {1}")]
    FileOpenError(String, String),
    #[error(
        "Unsupported texture with wrong width, height or color type."
    )]
    UnsupportedTextureError,
}
type Result<T> = std::result::Result<T, TextureError>;

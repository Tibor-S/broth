use std::{mem::size_of, ptr::copy_nonoverlapping as memcpy};

use thiserror::Error;
use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder},
    Device, Instance,
};

use crate::{
    command::{
        begin_single_time_commands, end_single_time_commands,
        CommandError,
    },
    memory::{get_memory_type_index, MemoryError},
};

pub type Mat3 = cgmath::Matrix3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            physical_device,
            properties,
            requirements,
        )?);

    // ! Do not do this for every buffer, (maybe use a memory pool)???
    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    swapchain_images: &[vk::Image],
    physical_device: vk::PhysicalDevice,
    camera_buffers: &mut Vec<vk::Buffer>,
    camera_buffers_memory: &mut Vec<vk::DeviceMemory>,
    model_buffers: &mut Vec<vk::Buffer>,
    model_buffers_memory: &mut Vec<vk::DeviceMemory>,
) -> Result<()> {
    camera_buffers.clear();
    model_buffers.clear();
    camera_buffers_memory.clear();
    model_buffers_memory.clear();

    for _ in 0..swapchain_images.len() {
        let (camera_buffer, camera_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size_of::<CameraObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        let (model_buffer, model_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size_of::<ModelObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        camera_buffers.push(camera_buffer);
        model_buffers.push(model_buffer);
        camera_buffers_memory.push(camera_buffer_memory);
        model_buffers_memory.push(model_buffer_memory);
    }

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraObject {
    pub view: Mat4,
    pub proj: Mat4,
    pub correction: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelObject {
    pub model: Mat4,
}

pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    graphics_queue: vk::Queue,
    physical_device: vk::PhysicalDevice,
    indices: &[u32],
    index_buffer: &mut vk::Buffer,
    index_buffer_memory: &mut vk::DeviceMemory,
    command_pool: vk::CommandPool,
) -> Result<()> {
    let size = (size_of::<u32>() * indices.len()) as u64;

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

    memcpy(indices.as_ptr(), memory.cast(), indices.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer_t, index_buffer_memory_t) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    *index_buffer = index_buffer_t;
    *index_buffer_memory = index_buffer_memory_t;

    copy_buffer(
        device,
        graphics_queue,
        command_pool,
        staging_buffer,
        *index_buffer,
        size,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn copy_buffer(
    device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer =
        begin_single_time_commands(device, command_pool)?;

    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(
        command_buffer,
        source,
        destination,
        &[regions],
    );

    end_single_time_commands(
        device,
        graphics_queue,
        command_pool,
        command_buffer,
    )?;

    Ok(())
}

#[derive(Debug, Error, Clone)]
pub enum BufferError {
    #[error(transparent)]
    MemoryError(#[from] MemoryError),
    #[error(transparent)]
    CommandError(#[from] CommandError),
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
}
type Result<T> = std::result::Result<T, BufferError>;

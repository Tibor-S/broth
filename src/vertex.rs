use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::vk::{self, DeviceV1_0, ErrorCode, HasBuilder};
use vulkanalia::{Device, Instance};

use crate::buffer::{copy_buffer, create_buffer, BufferError};
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    vertices: &[Vertex3],
    vertex_buffer: &mut vk::Buffer,
    vertex_buffer_memory: &mut vk::DeviceMemory,
) -> Result<()> {
    let size = (size_of::<Vertex3>() * vertices.len()) as u64;

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

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    (*vertex_buffer, *vertex_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(
        device,
        graphics_queue,
        command_pool,
        staging_buffer,
        *vertex_buffer,
        size,
    )?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_vertex_buffer_2d(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    vertices: &[Vertex2],
    vertex_buffer: &mut vk::Buffer,
    vertex_buffer_memory: &mut vk::DeviceMemory,
) -> Result<()> {
    let size = (size_of::<Vertex2>() * vertices.len()) as u64;

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

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    (*vertex_buffer, *vertex_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(
        device,
        graphics_queue,
        command_pool,
        staging_buffer,
        *vertex_buffer,
        size,
    )?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex3 {
    pub pos: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Vertex3 {
    #![allow(dead_code)]
    const fn new(pos: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self {
            pos,
            color,
            tex_coord,
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription
    {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex3>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions(
    ) -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32)
            .build();
        let tex_coord =
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(
                    (size_of::<Vec3>() + size_of::<Vec3>()) as u32,
                )
                .build();
        [pos, color, tex_coord]
    }
}
impl PartialEq for Vertex3 {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex3 {}

impl Hash for Vertex3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex2 {
    pub pos: Vec2,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Vertex2 {
    #![allow(dead_code)]
    const fn new(pos: Vec2, color: Vec3, tex_coord: Vec2) -> Self {
        Self {
            pos,
            color,
            tex_coord,
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription
    {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex2>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions(
    ) -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec2>() as u32)
            .build();
        let tex_coord =
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(
                    (size_of::<Vec2>() + size_of::<Vec3>()) as u32,
                )
                .build();
        [pos, color, tex_coord]
    }
}
impl PartialEq for Vertex2 {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex2 {}

impl Hash for Vertex2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum VertexError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    BufferError(#[from] BufferError),
}
#[derive(Debug, Clone, Copy)]
pub enum SpaceDimension {
    #[allow(dead_code)]
    D3,
    D2,
}
impl Default for SpaceDimension {
    fn default() -> Self {
        SpaceDimension::D3
    }
}
type Result<T> = std::result::Result<T, VertexError>;

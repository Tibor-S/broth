use std::fs::File;
use std::mem::size_of;

use crate::buffer::{
    create_index_buffer, create_uniform_buffers, BufferError, Mat4,
    UniformBufferObject,
};
use crate::color::{create_color_objects, ColorError};
use crate::command::{
    create_command_buffers, create_command_pool, CommandError,
};
use crate::descriptor::{
    create_descriptor_pool, create_descriptor_set_layout,
    create_descriptor_sets, DescriptorError,
};
use crate::device::{
    create_logical_device, pick_physical_device, DeviceError,
};
use crate::pipeline::{create_pipeline, PipelineError};
use crate::render_pass::{
    create_depth_objects, create_render_pass, RenderPassError,
};
use crate::swapchain::{
    create_framebuffers, create_swapchain,
    create_swapchain_image_views, create_sync_objects,
    SwapchainError,
};
use crate::texture::{
    create_texture_image, create_texture_image_view,
    create_texture_sampler, TextureError,
};
use crate::{
    instance::{create_instance, InstanceError},
    validation::destroy_debug_utils_messenger_ext,
    vertex::{create_vertex_buffer, Vertex, VertexError},
    MAX_FRAMES_IN_FLIGHT,
};
use cgmath::{point3, vec2, vec3, Deg};
use std::{
    collections::HashMap, io::BufReader,
    ptr::copy_nonoverlapping as memcpy, time::Instant,
};
use thiserror::Error;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    vk::{
        self, DeviceV1_0, Handle, HasBuilder, InstanceV1_0,
        KhrSurfaceExtension, KhrSwapchainExtension,
    },
    window::create_surface,
    Device, Entry, Instance,
};

use winit::window::Window;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    VkErrorCode(#[from] vk::ErrorCode),
    #[error(transparent)]
    LoadError(#[from] tobj::LoadError),
    #[error(transparent)]
    InstanceError(#[from] InstanceError),
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error(transparent)]
    SwapchainError(#[from] SwapchainError),
    #[error(transparent)]
    PipelineError(#[from] PipelineError),
    #[error(transparent)]
    DeviceError(#[from] DeviceError),
    #[error(transparent)]
    ColorError(#[from] ColorError),
    #[error(transparent)]
    TextureError(#[from] TextureError),
    #[error(transparent)]
    RenderPassError(#[from] RenderPassError),
    #[error(transparent)]
    DescriptorError(#[from] DescriptorError),
    #[error(transparent)]
    VertexError(#[from] VertexError),
    #[error(transparent)]
    CommandError(#[from] CommandError),
    #[error("Failed to open file with error: {0}.")]
    FileOpenError(String),
    #[error("{0:?}")]
    VkLibLoadingError(String),
    #[error("{0:?}")]
    VkEntryError(#[from] Box<dyn std::error::Error + Sync + Send>),
}
type Result<T> = std::result::Result<T, AppError>;

#[derive(Clone, Debug)]
pub struct App {
    pub _entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
}

#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub msaa_samples: vk::SampleCountFlags,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub mip_levels: u32,
    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY).map_err(|e| {
            AppError::VkLibLoadingError(e.to_string())
        })?;
        let entry = Entry::new(loader)?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = create_surface(&instance, &window, &window)?;
        pick_physical_device(&instance, &mut data)?;
        let device =
            create_logical_device(&entry, &instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        create_descriptor_set_layout(&device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_color_objects(&instance, &device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        create_texture_image(&instance, &device, &mut data)?;
        create_texture_image_view(&device, &mut data)?;
        create_texture_sampler(&device, &mut data)?;
        load_model(&mut data)?;
        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;
        Ok(Self {
            _entry: entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }

    unsafe fn update_uniform_buffer(
        &self,
        image_index: usize,
    ) -> Result<()> {
        // MVP

        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_axis_angle(
            vec3(0.0, 0.0, 1.0),
            Deg(90.0) * (time / 4f32),
        );

        let view = Mat4::look_at_rh(
            point3::<f32>(2.0, 2.0, 2.0),
            point3::<f32>(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );
        let correction = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            // We're also flipping the Y-axis with this line's `-1.0`.
            0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5,
            1.0,
        );

        let proj = correction
            * cgmath::perspective(
                Deg(45.0),
                self.data.swapchain_extent.width as f32
                    / self.data.swapchain_extent.height as f32,
                0.1,
                10.0,
            );

        let ubo = UniformBufferObject { model, view, proj };

        // Copy

        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(
            self.data.uniform_buffers_memory[image_index],
        );

        Ok(())
    }

    unsafe fn recreate_swapchain(
        &mut self,
        window: &Window,
    ) -> Result<()> {
        log::debug!("Recreating swapchain.");
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(
            window,
            &self.instance,
            &self.device,
            &mut self.data,
        )?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(
            &self.instance,
            &self.device,
            &mut self.data,
        )?;
        create_pipeline(&self.device, &mut self.data)?;
        create_color_objects(
            &self.instance,
            &self.device,
            &mut self.data,
        )?;
        create_depth_objects(
            &self.instance,
            &self.device,
            &mut self.data,
        )?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_uniform_buffers(
            &self.instance,
            &self.device,
            &mut self.data,
        )?;
        create_descriptor_pool(&self.device, &mut self.data)?;
        create_descriptor_sets(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        Ok(())
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.device.wait_for_fences(
            &[self.data.in_flight_fences[self.frame]],
            true,
            u64::MAX,
        )?;

        let image_index = match self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        ) {
            Ok((i, _)) => i as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                return self.recreate_swapchain(window);
            }
            Err(e) => return Err(e.into()),
        };

        if !self.data.images_in_flight[image_index as usize].is_null()
        {
            self.device.wait_for_fences(
                &[self.data.images_in_flight[image_index as usize]],
                true,
                u64::MAX,
            )?;
        }

        self.data.images_in_flight[image_index as usize] =
            self.data.in_flight_fences[self.frame];

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores =
            &[self.data.image_available_semaphores[self.frame]];
        let wait_stages =
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers =
            &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores =
            &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[
            self.data.in_flight_fences[self.frame]
        ])?;

        self.device.queue_submit(
            self.data.graphics_queue,
            &[submit_info],
            self.data.in_flight_fences[self.frame],
        )?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(
            self.data.present_queue,
            &present_info,
        );

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(e.into());
        }

        self.device.queue_wait_idle(self.data.present_queue)?;
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        log::debug!("Destroying application.");
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.data
            .in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data
            .render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data
            .image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));

        self.device.free_memory(self.data.index_buffer_memory, None);
        self.device.destroy_buffer(self.data.index_buffer, None);

        self.device
            .free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);

        self.device.destroy_sampler(self.data.texture_sampler, None);
        self.device
            .destroy_image_view(self.data.texture_image_view, None);
        self.device
            .free_memory(self.data.texture_image_memory, None);
        self.device.destroy_image(self.data.texture_image, None);

        self.device
            .destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_descriptor_set_layout(
            self.data.descriptor_set_layout,
            None,
        );

        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        destroy_debug_utils_messenger_ext(
            &self.instance,
            self.data.messenger,
        );

        self.instance.destroy_instance(None);
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.device.free_command_buffers(
            self.data.command_pool,
            &self.data.command_buffers,
        );

        self.device
            .destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data
            .uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        self.data
            .uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));

        self.device
            .destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);

        self.device
            .destroy_image_view(self.data.color_image_view, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image(self.data.color_image, None);

        self.data
            .framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));

        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);

        self.data
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}

fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = BufReader::new(
        File::open("resources/viking_room.obj").map_err(|e| {
            AppError::FileOpenError(format!(
                "Failed to open object with error: {}",
                e
            ))
        })?,
    );

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;
    let mut unique_vertices = HashMap::new();
    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertices.push(vertex);
                data.indices.push(index as u32);
            }
        }
    }
    Ok(())
}

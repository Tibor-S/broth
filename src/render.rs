use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode},
    Device,
};

use crate::{
    app::AppData,
    vertex::{Vertex2, Vertex3},
};

#[derive(Debug, Clone, Default)]
pub struct RenderObject {
    pub render_pass: vk::RenderPass,
    pub render_pass_2d: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline_layout_2d: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set_layout_2d: vk::DescriptorSetLayout,
    pub pipeline: vk::Pipeline,
    pub pipeline_2d: vk::Pipeline,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub command_buffers_2d: Vec<vk::CommandBuffer>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_2d: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub vertex_buffer_memory_2d: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_2d: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_buffer_memory_2d: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub render_dimension: RenderDimension,
}

impl RenderObject {
    pub fn command_buffer(&mut self, i: usize) -> vk::CommandBuffer {
        match self.render_dimension {
            RenderDimension::D2 => self.command_buffers_2d[i],
            RenderDimension::D3 => self.command_buffers[i],
        }
    }

    pub unsafe fn destroy_static(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        match self.render_dimension {
            RenderDimension::D2 => {
                self.destroy_static_2d(device, data)
            }
            RenderDimension::D3 => {
                self.destroy_static_3d(device, data)
            }
        }
    }

    unsafe fn destroy_static_2d(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        device.free_memory(self.index_buffer_memory_2d, None);
        device.destroy_buffer(self.index_buffer_2d, None);

        device.free_memory(self.vertex_buffer_memory_2d, None);
        device.destroy_buffer(self.vertex_buffer_2d, None);

        device.destroy_descriptor_set_layout(
            self.descriptor_set_layout_2d,
            None,
        );
    }
    unsafe fn destroy_static_3d(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        device.free_memory(self.index_buffer_memory, None);
        device.destroy_buffer(self.index_buffer, None);

        device.free_memory(self.vertex_buffer_memory, None);
        device.destroy_buffer(self.vertex_buffer, None);

        device.destroy_descriptor_set_layout(
            self.descriptor_set_layout,
            None,
        );
    }

    pub unsafe fn destroy_vars(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        match self.render_dimension {
            RenderDimension::D2 => self.destroy_vars_2d(device, data),
            RenderDimension::D3 => self.destroy_vars_3d(device, data),
        }
    }

    unsafe fn destroy_vars_2d(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        device.free_command_buffers(
            data.command_pool,
            &self.command_buffers_2d,
        );
        // self.uniform_buffers_memory
        //     .iter()
        //     .for_each(|m| device.free_memory(*m, None));
        // self.uniform_buffers
        //     .iter()
        //     .for_each(|b| device.destroy_buffer(*b, None));

        device.destroy_pipeline(self.pipeline_2d, None);
        device.destroy_pipeline_layout(self.pipeline_layout_2d, None);
        device.destroy_render_pass(self.render_pass_2d, None);
    }
    unsafe fn destroy_vars_3d(
        &self,
        device: &Device,
        data: &AppData,
    ) {
        device.free_command_buffers(
            data.command_pool,
            &self.command_buffers,
        );
        self.uniform_buffers_memory
            .iter()
            .for_each(|m| device.free_memory(*m, None));
        self.uniform_buffers
            .iter()
            .for_each(|b| device.destroy_buffer(*b, None));

        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_render_pass(self.render_pass, None);
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RenderObjectError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
}
#[derive(Debug, Clone, Copy, Default)]
pub enum RenderDimension {
    D2,
    #[default]
    D3,
}
type Result<T> = std::result::Result<T, RenderObjectError>;

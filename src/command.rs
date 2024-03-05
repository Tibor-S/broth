use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, Handle, HasBuilder},
    Device, Instance,
};

use crate::{
    app::AppData,
    queue::{QueueError, QueueFamilyIndices},
};

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(
        instance,
        data,
        data.physical_device,
    )?;
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // * Optional.
        .queue_family_index(indices.graphics);

    data.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

pub unsafe fn create_command_buffers(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    // Allocate

    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.render_object.command_buffers =
        device.allocate_command_buffers(&allocate_info)?;

    // Commands
    for (i, command_buffer) in
        data.render_object.command_buffers.iter().enumerate()
    {
        let info = vk::CommandBufferBeginInfo::builder();

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_object.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(
            *command_buffer,
            &info,
            vk::SubpassContents::INLINE,
        );
        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.render_object.pipeline,
        );
        device.cmd_bind_vertex_buffers(
            *command_buffer,
            0,
            &[data.render_object.vertex_buffer],
            &[0],
        );
        device.cmd_bind_index_buffer(
            *command_buffer,
            data.render_object.index_buffer,
            0,
            vk::IndexType::UINT32,
        );
        device.cmd_bind_descriptor_sets(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.render_object.pipeline_layout,
            0,
            &[data.descriptor_sets[i]],
            &[],
        );
        device.cmd_draw_indexed(
            *command_buffer,
            data.render_object.indices.len() as u32,
            1,
            0,
            0,
            0,
        );
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }
    log::debug!("!!!\n");
    Ok(())
}

pub unsafe fn create_command_buffers_2d(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    // Allocate

    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.render_object.command_buffers_2d =
        device.allocate_command_buffers(&allocate_info)?;

    // Commands
    for (i, command_buffer) in
        data.render_object.command_buffers_2d.iter().enumerate()
    {
        let info = vk::CommandBufferBeginInfo::builder();

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_object.render_pass_2d)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(
            *command_buffer,
            &info,
            vk::SubpassContents::INLINE,
        );
        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.render_object.pipeline_2d,
        );
        device.cmd_bind_vertex_buffers(
            *command_buffer,
            0,
            &[data.render_object.vertex_buffer_2d],
            &[0],
        );
        device.cmd_bind_index_buffer(
            *command_buffer,
            data.render_object.index_buffer_2d,
            0,
            vk::IndexType::UINT32,
        );
        device.cmd_bind_descriptor_sets(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.render_object.pipeline_layout_2d,
            0,
            &[data.descriptor_sets[i]],
            &[],
        );
        device.cmd_draw_indexed(
            *command_buffer,
            data.render_object.indices_2d.len() as u32,
            1,
            0,
            0,
            0,
        );
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}

pub unsafe fn begin_single_time_commands(
    device: &Device,
    data: &AppData,
) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    Ok(command_buffer)
}

pub unsafe fn end_single_time_commands(
    device: &Device,
    data: &AppData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info =
        vk::SubmitInfo::builder().command_buffers(command_buffers);

    device.queue_submit(
        data.graphics_queue,
        &[info],
        vk::Fence::null(),
    )?;
    device.queue_wait_idle(data.graphics_queue)?;

    device.free_command_buffers(data.command_pool, &[command_buffer]);

    Ok(())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    QueueError(#[from] QueueError),
}
type Result<T> = std::result::Result<T, CommandError>;

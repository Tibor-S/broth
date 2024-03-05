use std::mem::size_of;

use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder},
    Device,
};

use crate::{app::AppData, buffer::UniformBufferObject};

pub unsafe fn create_descriptor_set_layout(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);

    data.render_object.descriptor_set_layout =
        device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_set_layout_2d(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);

    data.render_object.descriptor_set_layout_2d =
        device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_pool(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32);

    data.descriptor_pool =
        device.create_descriptor_pool(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_sets(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    // Allocate

    let layouts = vec![
        data.render_object.descriptor_set_layout;
        data.swapchain_images.len()
    ];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.render_object.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_image_view)
            .sampler(data.texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            )
            .image_info(image_info);

        device.update_descriptor_sets(
            &[ubo_write, sampler_write],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}

pub unsafe fn create_descriptor_sets_2d(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    // Allocate

    let layouts = vec![
        data.render_object.descriptor_set_layout_2d;
        data.swapchain_images.len()
    ];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..data.swapchain_images.len() {
        // let info = vk::DescriptorBufferInfo::builder()
        //     .buffer(data.uniform_buffers[i])
        //     .offset(0)
        //     .range(size_of::<UniformBufferObject>() as u64);

        // let buffer_info = &[info];
        // let ubo_write = vk::WriteDescriptorSet::builder()
        //     .dst_set(data.descriptor_sets[i])
        //     .dst_binding(0)
        //     .dst_array_element(0)
        //     .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        //     .buffer_info(buffer_info);
        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_image_view)
            .sampler(data.texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            )
            .image_info(image_info);

        device.update_descriptor_sets(
            &[sampler_write],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DescriptorError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
}
type Result<T> = std::result::Result<T, DescriptorError>;

use std::mem::size_of;

use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder},
    Device,
};

use crate::buffer::{CameraObject, ModelObject};

pub unsafe fn create_descriptor_set_layout(
    device: &Device,
    descriptor_set_layout: &mut vk::DescriptorSetLayout,
    uniform_buffer_count: u32,
) -> Result<()> {
    let mut bindings = vec![];

    for i in 0..uniform_buffer_count {
        let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(i)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        bindings.push(ubo_binding);
    }

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(2)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    bindings.push(sampler_binding);

    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings);

    *descriptor_set_layout =
        device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_pool(
    device: &Device,
    swapchain_images_len: u32,
    uniform_buffer_count: u32,
    descriptor_pool: &mut vk::DescriptorPool,
) -> Result<()> {
    let mut pool_sizes = vec![];

    for _ in 0..uniform_buffer_count {
        let ubo_size = vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_images_len);

        pool_sizes.push(ubo_size);
    }
    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(swapchain_images_len);

    pool_sizes.push(sampler_size);
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(swapchain_images_len);

    *descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_sets(
    device: &Device,
    swapchain_images_len: usize,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    camera_buffers: &[vk::Buffer],
    model_buffers: &[vk::Buffer],
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    descriptor_sets: &mut Vec<vk::DescriptorSet>,
) -> Result<()> {
    // Allocate

    let layouts = vec![descriptor_set_layout; swapchain_images_len];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    *descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..swapchain_images_len {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(camera_buffers[i])
            .offset(0)
            .range(size_of::<CameraObject>() as u64);

        let buffer_info = &[info];
        let camera_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(model_buffers[i])
            .offset(0)
            .range(size_of::<ModelObject>() as u64);

        let buffer_info = &[info];
        let model_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(2)
            .dst_array_element(0)
            .descriptor_type(
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            )
            .image_info(image_info);

        device.update_descriptor_sets(
            &[camera_write, model_write, sampler_write],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}

pub unsafe fn create_descriptor_sets_2d(
    device: &Device,
    swapchain_images_len: usize,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniform_buffers: &[vk::Buffer],
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    descriptor_sets: &mut Vec<vk::DescriptorSet>,
) -> Result<()> {
    // Allocate

    let layouts = vec![descriptor_set_layout; swapchain_images_len];

    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    *descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..swapchain_images_len {
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
            .image_view(texture_image_view)
            .sampler(texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
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

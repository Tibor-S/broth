use log::{error, info};
use thiserror::Error;
use vulkanalia::vk::{self, ErrorCode, HasBuilder};
use vulkanalia::{Entry, Instance};
use winit::window::Window;

use crate::validation::{
    validated_extensions, validated_info, validated_instance,
    validated_layers, ValidationError,
};
use crate::PORTABILITY_MACOS_VERSION;

pub unsafe fn create_instance(
    window: &Window,
    entry: &Entry,
    messenger: &mut vk::DebugUtilsMessengerEXT,
) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Broth\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let layers = validated_layers(entry)?;

    let mut extensions = validated_extensions(window)?;

    // Required by Vulkan SDK on macOS since 1.3.216.
    let flags = if cfg!(target_os = "macos")
        && entry.version()? >= PORTABILITY_MACOS_VERSION
    {
        info!("Enabling extensions for macOS portability.");
        extensions.push(
            vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                .name
                .as_ptr(),
        );
        extensions.push(
            vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr(),
        );
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    let (info, _debug_info) = validated_info(
        &application_info,
        &layers,
        &extensions,
        flags,
    )?;

    let instance = validated_instance(entry, &info, messenger)?;
    Ok(instance)
}

#[derive(Debug, Error, Clone)]
pub enum InstanceError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
}
type Result<T> = std::result::Result<T, InstanceError>;

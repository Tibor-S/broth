use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;
use thiserror::Error;

use vulkanalia::vk::{
    self, DebugUtilsMessengerCreateInfoEXTBuilder, EntryV1_0,
    ErrorCode, ExtDebugUtilsExtension, HasBuilder,
};
use vulkanalia::Entry;
use vulkanalia::{window as vk_window, Instance};
use winit::window::Window;

use crate::app::AppData;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

#[derive(Clone, Debug, Error)]
pub enum ValidationError {
    #[error("Validation layer requested but not supported.")]
    NoSupport,
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
}
type Result<T> = std::result::Result<T, ValidationError>;

pub unsafe fn validated_layers(
    entry: &Entry,
) -> Result<Vec<*const i8>> {
    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED
        && !available_layers.contains(&VALIDATION_LAYER)
    {
        return Err(ValidationError::NoSupport);
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    Ok(layers)
}

pub fn validated_extensions(
    window: &Window,
) -> Result<Vec<*const i8>> {
    let mut extensions =
        vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    Ok(extensions)
}

pub unsafe fn validated_instance(
    entry: &Entry,
    info: &vk::InstanceCreateInfo,
    data: &mut AppData,
) -> Result<Instance> {
    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        data.messenger = instance
            .create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

pub unsafe fn validated_info<'a>(
    application_info: &vk::ApplicationInfo,
    layers: &Vec<*const i8>,
    extensions: &Vec<*const i8>,
    flags: vk::InstanceCreateFlags,
) -> Result<(
    vk::InstanceCreateInfo,
    DebugUtilsMessengerCreateInfoEXTBuilder<'a>,
)> {
    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(application_info)
        .enabled_layer_names(layers)
        .enabled_extension_names(extensions)
        .flags(flags);

    let mut debug_info =
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::all(),
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    Ok((*info, debug_info))
}

pub unsafe fn destroy_debug_utils_messenger_ext(
    instance: &Instance,
    messenger: vk::DebugUtilsMessengerEXT,
) {
    if VALIDATION_ENABLED {
        instance.destroy_debug_utils_messenger_ext(messenger, None);
    }
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message =
        unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        log::error!("({:?}) {}", type_, message);
    } else if severity
        >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
    {
        log::warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO
    {
        log::debug!("({:?}) {}", type_, message);
    } else {
        log::trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

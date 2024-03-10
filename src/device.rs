use std::collections::HashSet;

use vulkanalia::{
    vk::{self, DeviceV1_0, ErrorCode, HasBuilder, InstanceV1_0},
    Device, Entry, Instance,
};

use crate::{
    queue::{QueueError, QueueFamilyIndices},
    swapchain::{SwapchainError, SwapchainSupport},
    validation::{validated_layers, ValidationError},
    DEVICE_EXTENSIONS, IS_MACOS, PORTABILITY_MACOS_VERSION,
};

pub unsafe fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    graphics_queue: &mut vk::Queue,
    present_queue: &mut vk::Queue,
) -> Result<Device> {
    let indices =
        QueueFamilyIndices::get(instance, surface, physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = validated_layers(entry)?;
    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    if IS_MACOS && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions
            .push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device =
        instance.create_device(physical_device, &info, None)?;
    *graphics_queue = device.get_device_queue(indices.graphics, 0);
    *present_queue = device.get_device_queue(indices.present, 0);

    Ok(device)
}

pub unsafe fn check_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, surface, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support =
        SwapchainSupport::get(instance, surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty()
    {
        return Err(DeviceError::SwapchainSupportError);
    }
    let features =
        instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(DeviceError::FeatureError(
            "No sampler anisotropy.".into(),
        ));
    }

    Ok(())
}

pub unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(DeviceError::MissingExtensions)
    }
}

pub unsafe fn pick_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: &mut vk::PhysicalDevice,
    msaa_samples: &mut vk::SampleCountFlags,
) -> Result<()> {
    for physical_device_t in instance.enumerate_physical_devices()? {
        let properties = instance
            .get_physical_device_properties(physical_device_t);

        match check_physical_device(
            instance,
            surface,
            physical_device_t,
        ) {
            Err(e) => {
                log::warn!(
                    "Skipping physical device (`{}`): {}",
                    properties.device_name,
                    e
                );
            }
            Ok(()) => {
                log::info!(
                    "Selected physical device (`{}`).",
                    properties.device_name
                );
                *physical_device = physical_device_t;
                *msaa_samples =
                    get_max_msaa_samples(instance, *physical_device);
                log::info!(
                    "Using msaa x{}",
                    match *msaa_samples {
                        vk::SampleCountFlags::_1 => 1,
                        vk::SampleCountFlags::_2 => 2,
                        vk::SampleCountFlags::_4 => 4,
                        vk::SampleCountFlags::_8 => 8,
                        vk::SampleCountFlags::_16 => 16,
                        vk::SampleCountFlags::_32 => 32,
                        vk::SampleCountFlags::_64 => 64,
                        _ => 1,
                    }
                );
                return Ok(());
            }
        }
        // * let features = instance.get_physical_device_features(physical_device);
    }

    // ************************
    // * Example of gpu tests *
    // ************************
    // * if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
    // *    return Err(RootError::SuitabilityError("Only discrete GPUs are supported."));
    // * }
    // * if features.geometry_shader != vk::TRUE {
    // *     return Err(RootError::SuitabilityError("Missing geometry shader support."));
    // * }

    Ok(())
}

unsafe fn get_max_msaa_samples(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let properties =
        instance.get_physical_device_properties(physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DeviceError {
    #[error(transparent)]
    VkErrorCode(#[from] ErrorCode),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error(transparent)]
    QueueError(#[from] QueueError),
    #[error(transparent)]
    SwapchainError(#[from] SwapchainError),
    #[error("Missing required device extensions.")]
    MissingExtensions,
    #[error("Insufficient swapchain support.")]
    SwapchainSupportError,
    #[error("Insufficient swapchain support.")]
    FeatureError(String),
}
type Result<T> = std::result::Result<T, DeviceError>;

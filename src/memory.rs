use vulkanalia::{
    vk::{self, InstanceV1_0},
    Instance,
};

pub unsafe fn get_memory_type_index(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = instance
        .get_physical_device_memory_properties(physical_device);
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable =
                (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable
                && memory_type.property_flags.contains(properties)
        })
        .ok_or(MemoryError::SuitabilityError)
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MemoryError {
    #[error("No suitable memory type found.")]
    SuitabilityError,
}
type Result<T> = std::result::Result<T, MemoryError>;

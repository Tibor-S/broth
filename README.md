# Broth

## TO DO

- Create explicit queue family for transfering buffers.
  - https://kylemayes.github.io/vulkanalia/vertex/staging_buffer.html
- Read through:
  - https://developer.nvidia.com/vulkan-memory-management

## VulkanSDK

https://vulkan.lunarg.com/sdk/home

## MacOS

```Shell
export VK_LAYER_PATH=/Users/USERNAME/VulkanSDK/x.x.xxx.xmacOS/share/vulkan/explicit_layers.d
export DYLD_LIBRARY_PATH=/Users/USERNAME/VulkanSDK/x.x.xxx.xmacOS/lib/
export DYLD_LIBRARY_PATH=/Users/USERNAME/VulkanSDK/x.x.xxx.x/macOS/lib/
export VK_ICD_FILENAMES=/Users/USERNAME/VulkanSDK/x.x.xxx.x/macOS/share/vulkan/icd.d/MoltenVK_icd.json
```

#!/usr/bin/env bash

# Check if vkconfig is running
if [ -z $(pidof vkconfig) ]
then
    echo "[ERROR] vkconfig is not running. Required for validation layers."
    exit 1
fi

export RUST_LOG=debug
export RUST_BACKTRACE=1
export VULKAN_SDK=/Users/sebastian/VulkanSDK/1.3.268.1/macOS
export VK_LOADER_LAYERS_ENABLE=*validation,*profiles

export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layers.d
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export DYLD_LIBRARY_PATH=$VULKAN_SDK/lib/

# Compile shaders

cargo run 
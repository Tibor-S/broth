#!/usr/bin/env bash

export RUST_LOG=debug
export VULKAN_SDK=/Users/sebastian/VulkanSDK/1.3.268.1/macOS
export VK_LOADER_LAYERS_ENABLE=*validation,*profiles


export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layers.d
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export DYLD_LIBRARY_PATH=$VULKAN_SDK/lib/

cargo build --release > .out 2> .log
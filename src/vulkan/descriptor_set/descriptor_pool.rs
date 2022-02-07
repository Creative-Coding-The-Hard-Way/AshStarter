use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};

use crate::vulkan::{
    descriptor_set::{DescriptorSet, DescriptorSetError, DescriptorSetLayout},
    errors::VulkanDebugError,
    RenderDevice, VulkanDebug,
};

/// An owned Descriptor Pool which is automatically destroyed when dropped.
pub struct DescriptorPool {
    /// The raw vulkan descriptor pool handle.
    pub raw: vk::DescriptorPool,

    /// The device used to create the pool.
    pub vk_dev: Arc<RenderDevice>,
}

impl DescriptorPool {
    /// Create a new descriptor pool with capacity for `descriptor_count`
    /// descritpors.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        descriptor_count: u32,
        sizes: &[vk::DescriptorPoolSize],
    ) -> Result<Self, DescriptorSetError> {
        let create_info = vk::DescriptorPoolCreateInfo {
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: descriptor_count,
            pool_size_count: sizes.len() as u32,
            p_pool_sizes: sizes.as_ptr(),
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_descriptor_pool(&create_info, None)
                .map_err(DescriptorSetError::UnableToCreatePool)?
        };
        Ok(Self { raw, vk_dev })
    }

    /// Create a new descriptor pool with capacity for one descriptor per
    /// swapchain image.
    pub fn for_each_swapchain_image(
        vk_dev: Arc<RenderDevice>,
        uniform_buffers: u32,
        storage_buffers: u32,
        image_samplers: u32,
    ) -> Result<Self, DescriptorSetError> {
        let descriptor_count = vk_dev.swapchain_image_count();
        let mut sizes = vec![];
        if uniform_buffers > 0 {
            sizes.push(vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: uniform_buffers,
            });
        }
        if storage_buffers > 0 {
            sizes.push(vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: storage_buffers,
            });
        }
        if image_samplers > 0 {
            sizes.push(vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: image_samplers,
            });
        }
        Self::new(vk_dev, descriptor_count, &sizes)
    }

    /// Allocate descriptor sets from this pool.
    pub fn allocate(
        &self,
        layout: &DescriptorSetLayout,
        count: u32,
    ) -> Result<Vec<DescriptorSet>, DescriptorSetError> {
        let mut layouts = vec![];
        for _ in 0..count {
            layouts.push(layout.raw);
        }
        let allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.raw,
            descriptor_set_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };
        let raw_sets = unsafe {
            self.vk_dev
                .logical_device
                .allocate_descriptor_sets(&allocate_info)
                .map_err(DescriptorSetError::UnableToAllocateDescriptors)?
        };
        let descriptor_sets: Vec<DescriptorSet> = raw_sets
            .into_iter()
            .map(|raw| DescriptorSet {
                raw,
                vk_dev: self.vk_dev.clone(),
            })
            .collect();
        Ok(descriptor_sets)
    }
}

impl VulkanDebug for DescriptorPool {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_POOL,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for DescriptorPool {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_descriptor_pool(self.raw, None);
        }
    }
}

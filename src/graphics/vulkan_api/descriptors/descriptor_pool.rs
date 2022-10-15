use {
    crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError},
    ash::vk,
    std::sync::Arc,
};

/// An owned Vulkan Descriptor Pool.
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    render_device: Arc<RenderDevice>,
}

impl DescriptorPool {
    /// Create a new descriptor pool.
    pub fn new(
        render_device: Arc<RenderDevice>,
        pool_sizes: &[vk::DescriptorPoolSize],
        max_sets: u32,
    ) -> Result<Self, VulkanError> {
        let create_info = vk::DescriptorPoolCreateInfo {
            max_sets,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };
        let descriptor_pool =
            unsafe { render_device.create_descriptor_pool(&create_info)? };
        Ok(Self {
            descriptor_pool,
            render_device,
        })
    }

    /// Allocate a raw Vulkan descriptor set from this pool.
    ///
    /// # Safety
    ///
    /// Unsafe because the pool must not be destroyed while descriptor sets
    /// which use it still exist.
    pub unsafe fn allocate_descriptor_sets(
        &self,
        descriptor_set_count: u32,
        descriptor_layout: vk::DescriptorSetLayout,
    ) -> Result<Vec<vk::DescriptorSet>, VulkanError> {
        let layouts: Vec<vk::DescriptorSetLayout> = (0..descriptor_set_count)
            .map(|_| descriptor_layout)
            .collect();
        let create_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.descriptor_pool,
            p_set_layouts: layouts.as_ptr(),
            descriptor_set_count,
            ..Default::default()
        };
        self.render_device.allocate_descriptor_sets(&create_info)
    }
}

impl VulkanDebug for DescriptorPool {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_POOL,
            self.descriptor_pool,
        )
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .destroy_descriptor_pool(self.descriptor_pool)
        }
    }
}

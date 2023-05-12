use {
    crate::graphics::{
        vulkan_api::{raii, RenderDevice},
        GraphicsError,
    },
    ash::vk,
    std::sync::Arc,
};

/// RAII Vulkan DescriptorPool.
pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    render_device: Arc<RenderDevice>,
}

impl DescriptorPool {
    /// Create a new Vulkan descriptor pool.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - command pools must be destroyed before the Vulkan device is dropped.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::DescriptorPoolCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let descriptor_pool = unsafe {
            render_device
                .device()
                .create_descriptor_pool(create_info, None)?
        };
        Ok(Self {
            descriptor_pool,
            descriptor_sets: vec![],
            render_device,
        })
    }

    /// Create a new Vulkan descriptor pool using the max_sets and pool_sizes.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - command pools must be destroyed before the Vulkan device is dropped.
    pub unsafe fn new_with_sizes(
        render_device: Arc<RenderDevice>,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> Result<Self, GraphicsError> {
        let create_info = vk::DescriptorPoolCreateInfo {
            max_sets,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..vk::DescriptorPoolCreateInfo::default()
        };
        Self::new(render_device, &create_info)
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.descriptor_pool,
            vk::ObjectType::COMMAND_POOL,
            name,
        );
    }

    /// Get the n'th descriptor set owned by this pool.
    ///
    /// Note: The descriptor pool destroys all allocated sets when it is
    /// dropped. The caller must ensure that no descriptor sets are kept around
    /// after the pool is dropped.
    pub fn descriptor_set(&self, index: usize) -> vk::DescriptorSet {
        self.descriptor_sets[index]
    }

    /// Allocate descriptor sets from this pool.
    ///
    /// # Returns
    ///
    /// Returns the index of the first newly allocated descriptor set.
    pub fn allocate_descriptor_sets(
        &mut self,
        layouts: &[&raii::DescriptorSetLayout],
    ) -> Result<usize, GraphicsError> {
        let descriptor_set_count = layouts.len() as u32;
        let raw_layouts: Vec<vk::DescriptorSetLayout> =
            layouts.iter().map(|layout| layout.raw()).collect();

        let create_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.descriptor_pool,
            descriptor_set_count,
            p_set_layouts: raw_layouts.as_ptr(),
            ..vk::DescriptorSetAllocateInfo::default()
        };
        let descriptor_sets = unsafe {
            self.render_device
                .device()
                .allocate_descriptor_sets(&create_info)?
        };
        let last = self.descriptor_sets.len();
        self.descriptor_sets.extend_from_slice(&descriptor_sets);
        Ok(last)
    }

    /// Get the raw Vulkan command pool handle.
    pub fn raw(&self) -> vk::DescriptorPool {
        self.descriptor_pool
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

impl std::fmt::Debug for DescriptorPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorPool")
            .field("descriptor_pool", &self.descriptor_pool)
            .field("descriptor_sets", &self.descriptor_sets)
            .finish()
    }
}

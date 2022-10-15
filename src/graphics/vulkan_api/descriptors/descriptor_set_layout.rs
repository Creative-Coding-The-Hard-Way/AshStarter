use {
    crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError},
    ash::vk,
    std::sync::Arc,
};

/// An owned Vulkan descriptor set layout.
pub struct DescriptorSetLayout {
    descriptor_set_layout: vk::DescriptorSetLayout,
    render_device: Arc<RenderDevice>,
}

impl DescriptorSetLayout {
    /// Create a new descriptor set layout.
    pub fn new(
        render_device: Arc<RenderDevice>,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<Self, VulkanError> {
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        };
        let descriptor_set_layout = unsafe {
            render_device.create_descriptor_set_layout(&create_info)?
        };
        Ok(Self {
            descriptor_set_layout,
            render_device,
        })
    }

    /// Get the raw Vulkan handle for the descriptor set layout.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - ownership is not transferred
    ///   - the caller must ensure the handle does not outlive this instance.
    pub unsafe fn raw(&self) -> &vk::DescriptorSetLayout {
        &self.descriptor_set_layout
    }
}

impl VulkanDebug for DescriptorSetLayout {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
            self.descriptor_set_layout,
        );
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .destroy_descriptor_set_layout(self.descriptor_set_layout)
        }
    }
}

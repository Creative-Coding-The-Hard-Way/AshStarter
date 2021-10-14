use super::{DescriptorSetError, DescriptorSetLayout};

use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl DescriptorSetLayout {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<Self, DescriptorSetError> {
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            p_bindings: bindings.as_ptr(),
            binding_count: bindings.len() as u32,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(DescriptorSetError::UnableToCreateLayout)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl VulkanDebug for DescriptorSetLayout {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for DescriptorSetLayout {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_descriptor_set_layout(self.raw, None);
        }
    }
}

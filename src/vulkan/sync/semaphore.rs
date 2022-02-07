use ::{ash::vk, std::sync::Arc};

use crate::vulkan::{
    errors::VulkanDebugError, sync::SemaphoreError, RenderDevice, VulkanDebug,
};

/// An owned semaphore which is automatically destroyed when it is dropped.
pub struct Semaphore {
    pub raw: vk::Semaphore,
    pub vk_dev: Arc<RenderDevice>,
}

impl Semaphore {
    /// Create a new semaphore.
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, SemaphoreError> {
        let create_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_semaphore(&create_info, None)
                .map_err(SemaphoreError::UnableToCreateSemaphore)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl VulkanDebug for Semaphore {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::SEMAPHORE,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for Semaphore {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_semaphore(self.raw, None);
        }
    }
}

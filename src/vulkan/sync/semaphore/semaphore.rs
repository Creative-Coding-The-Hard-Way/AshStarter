use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

use super::{Semaphore, SemaphoreError};
use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

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

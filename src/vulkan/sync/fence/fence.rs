use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

use super::{Fence, FenceError};
use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

impl Fence {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, FenceError> {
        let raw = {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_fence(&create_info, None)
                    .map_err(FenceError::UnableToCreateFence)?
            }
        };
        Ok(Self { raw, vk_dev })
    }

    /// Block until the fence is signalled, then reset.
    pub fn wait_and_reset(&self) -> Result<(), FenceError> {
        self.wait()?;
        self.reset()
    }

    /// Block until the fence is signaled.
    pub fn wait(&self) -> Result<(), FenceError> {
        unsafe {
            self.vk_dev
                .logical_device
                .wait_for_fences(&[self.raw], true, u64::MAX)
                .map_err(FenceError::UnexpectedWaitError)?;
        }
        Ok(())
    }

    /// Reset the fence for future signalling.
    pub fn reset(&self) -> Result<(), FenceError> {
        unsafe {
            self.vk_dev
                .logical_device
                .reset_fences(&[self.raw])
                .map_err(FenceError::UnexpectedResetError)?;
        }
        Ok(())
    }
}

impl VulkanDebug for Fence {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::FENCE,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for Fence {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_fence(self.raw, None);
        }
    }
}

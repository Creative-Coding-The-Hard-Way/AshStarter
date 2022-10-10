use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError};

/// An owned Vulkan fence object which is automatically destroyed when dropped.
pub struct Fence {
    /// The raw fence handle.
    raw: vk::Fence,

    /// The device which created the fence.
    render_device: Arc<RenderDevice>,
}

impl Fence {
    /// Create a new Fence in the SIGNALED state.
    pub fn new(render_device: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let raw = {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            unsafe { render_device.create_fence(&create_info)? }
        };
        Ok(Self { raw, render_device })
    }

    /// Get the raw Vulkan resource handle.
    ///
    /// # Safety
    ///
    /// Ownership is *not* transfered. It is the responsibility of the caller
    /// to ensure the underlying resource handle is not kept beyond the
    /// lifetime of this Fence instance.
    pub unsafe fn raw(&self) -> &vk::Fence {
        &self.raw
    }

    /// Block until the fence is signalled, then reset.
    pub fn wait_and_reset(&self) -> Result<(), VulkanError> {
        self.wait()?;
        self.reset()
    }

    /// Block until the fence is signaled.
    pub fn wait(&self) -> Result<(), VulkanError> {
        self.render_device.wait_for_fences(&[self.raw], true)
    }

    /// Reset the fence for future signalling.
    pub fn reset(&self) -> Result<(), VulkanError> {
        self.render_device.reset_fences(&[self.raw])
    }

    /// Check if the fence has been signalled.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - A fence's status can be immediately out of date if a queue is
    ///     pending submission.
    pub unsafe fn get_status(&self) -> Result<bool, VulkanError> {
        self.render_device.get_fence_status(self.raw)
    }
}

impl VulkanDebug for Fence {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::FENCE,
            self.raw,
        )
    }
}

impl Drop for Fence {
    /// # Safety
    ///
    /// The application must ensure that this Fence is not in-use by the GPU
    /// when it is dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_fence(self.raw);
        }
    }
}

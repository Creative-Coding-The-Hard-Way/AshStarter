use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanError};

/// An owned Vulkan image sampler.
pub struct Sampler {
    sampler: vk::Sampler,
    render_device: Arc<RenderDevice>,
}

impl Sampler {
    /// Create a new Vulkan sampler object.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::SamplerCreateInfo,
    ) -> Result<Self, VulkanError> {
        let sampler = unsafe { render_device.create_sampler(create_info)? };
        Ok(Self {
            sampler,
            render_device,
        })
    }

    /// Get the raw Vulkan sampler handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - ownership is not transferred.
    pub unsafe fn raw(&self) -> &vk::Sampler {
        &self.sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_sampler(self.sampler);
        }
    }
}

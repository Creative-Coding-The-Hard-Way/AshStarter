use {
    crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError},
    ash::vk,
    std::sync::Arc,
};

/// An owned Vulkan semaphore object which is automatically destroyed when
/// dropped.
pub struct Semaphore {
    /// The raw fence handle.
    raw: vk::Semaphore,

    /// The device which created the fence.
    render_device: Arc<RenderDevice>,
}

impl Semaphore {
    /// Create a new semaphore.
    pub fn new(render_device: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let create_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let raw = unsafe { render_device.create_semaphore(&create_info)? };
        Ok(Self { raw, render_device })
    }

    /// Get the underlying Vulkan resource handle.
    ///
    /// # Safety
    ///
    /// Ownership is not transfered. The caller is responsible for ensuring that
    /// the handle is not kept beyond the lifetime of this object.
    pub unsafe fn raw(&self) -> &vk::Semaphore {
        &self.raw
    }
}

impl VulkanDebug for Semaphore {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::SEMAPHORE,
            self.raw,
        );
    }
}

impl Drop for Semaphore {
    /// # Safety
    ///
    /// The application must ensure the Semaphore is no longer in use by the GPU
    /// before it is dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_semaphore(self.raw);
        }
    }
}

use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

/// RAII Vulkan Semaphore.
pub struct Semaphore {
    semaphore: vk::Semaphore,
    render_device: Arc<RenderDevice>,
}

impl Semaphore {
    /// Create a new Vulkan semaphore.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The semaphore must be dropped before the render device.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
    ) -> Result<Self, GraphicsError> {
        let create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe {
            render_device
                .device()
                .create_semaphore(&create_info, None)?
        };
        Ok(Self {
            semaphore,
            render_device,
        })
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.semaphore,
            vk::ObjectType::SEMAPHORE,
            name,
        );
    }

    /// Get the Vulkan semaphore handle.
    pub fn raw(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}

impl std::fmt::Debug for Semaphore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Semaphore")
            .field("semaphore", &self.semaphore)
            .finish()
    }
}

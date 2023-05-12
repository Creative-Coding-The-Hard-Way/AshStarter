use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

/// RAII Vulkan Fence.
pub struct Fence {
    fence: vk::Fence,
    render_device: Arc<RenderDevice>,
}

impl Fence {
    /// Create a new Vulkan fence.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The fence must be dropped before the render device.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::FenceCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let fence =
            unsafe { render_device.device().create_fence(create_info, None)? };
        Ok(Self {
            fence,
            render_device,
        })
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.fence,
            vk::ObjectType::FENCE,
            name,
        );
    }

    /// Get the Vulkan fence handle.
    pub fn raw(&self) -> vk::Fence {
        self.fence
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.render_device.device().destroy_fence(self.fence, None);
        }
    }
}

impl std::fmt::Debug for Fence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fence").field("fence", &self.fence).finish()
    }
}

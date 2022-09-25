use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError};

/// An owned Vulkan pipeline which is automatically destroyed when dropped.
pub struct PipelineLayout {
    pipeline_layout: vk::PipelineLayout,
    render_device: Arc<RenderDevice>,
}

impl PipelineLayout {
    /// Create a new Pipeline Layout.
    pub fn new(render_device: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let create_info = vk::PipelineLayoutCreateInfo {
            ..Default::default()
        };
        let pipeline_layout =
            unsafe { render_device.create_pipeline_layout(&create_info)? };
        Ok(Self {
            pipeline_layout,
            render_device,
        })
    }

    /// Get the raw Vulkan pipeline layout handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Ownership is not transferred. The caller is responsible for not
    ///     retaining any copies of the handle once this PipelineLayout
    ///     instance is dropped.
    pub unsafe fn raw(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

impl VulkanDebug for PipelineLayout {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::PIPELINE_LAYOUT,
            self.pipeline_layout,
        );
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .destroy_pipeline_layout(self.pipeline_layout);
        };
    }
}

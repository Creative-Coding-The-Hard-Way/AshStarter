use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError};

/// A Vulkan graphics pipeline and associated information.
pub struct GraphicsPipeline {
    pipeline: vk::Pipeline,
    render_device: Arc<RenderDevice>,
}

impl GraphicsPipeline {
    /// Create a new owned graphics pipeline.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::GraphicsPipelineCreateInfo,
    ) -> Result<Self, VulkanError> {
        let pipeline =
            unsafe { render_device.create_graphics_pipeline(create_info)? };
        Ok(Self {
            pipeline,
            render_device,
        })
    }

    /// Get the raw Vulkan pipeline handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Ownership is not transferred. The application must not drop the
    ///     GraphicsPipeline instance until all GPU operations which reference
    ///     the pipeline are finished.
    pub unsafe fn raw(&self) -> &vk::Pipeline {
        &self.pipeline
    }
}

impl VulkanDebug for GraphicsPipeline {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::PIPELINE,
            self.pipeline,
        );
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe { self.render_device.destroy_pipeline(self.pipeline) }
    }
}

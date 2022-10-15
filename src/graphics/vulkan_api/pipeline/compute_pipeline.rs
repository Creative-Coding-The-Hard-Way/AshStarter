use {
    crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError},
    ash::vk,
    std::sync::Arc,
};

/// A Vulkan compute pipeline and associated information.
pub struct ComputePipeline {
    pipeline: vk::Pipeline,
    render_device: Arc<RenderDevice>,
}

impl ComputePipeline {
    /// Create a new owned compute pipeline.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::ComputePipelineCreateInfo,
    ) -> Result<Self, VulkanError> {
        let pipeline =
            unsafe { render_device.create_compute_pipeline(create_info)? };
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
    ///     ComputePipeline instance until all GPU operations which reference
    ///     the pipeline are finished.
    pub unsafe fn raw(&self) -> &vk::Pipeline {
        &self.pipeline
    }
}

impl VulkanDebug for ComputePipeline {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::PIPELINE,
            self.pipeline,
        );
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe { self.render_device.destroy_pipeline(self.pipeline) }
    }
}

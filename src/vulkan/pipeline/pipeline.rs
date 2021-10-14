use super::{Pipeline, PipelineError};

use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

use {
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl Pipeline {
    /// Create a new graphics pipeline.
    pub fn new_graphics_pipeline(
        vk_dev: Arc<RenderDevice>,
        create_info: vk::GraphicsPipelineCreateInfo,
    ) -> Result<Pipeline, PipelineError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[create_info],
                    None,
                )
                .map_err(|(_, err)| {
                    PipelineError::UnableToCreateGraphicsPipeline(err)
                })?[0]
        };
        Ok(Self {
            raw,
            bind_point: vk::PipelineBindPoint::GRAPHICS,
            vk_dev,
        })
    }
}

impl VulkanDebug for Pipeline {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::PIPELINE,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for Pipeline {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_pipeline(self.raw, None);
        }
    }
}

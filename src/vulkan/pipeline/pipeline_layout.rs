use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};

use crate::vulkan::{
    errors::VulkanDebugError, pipeline::PipelineError, RenderDevice,
    VulkanDebug,
};

/// An owned Pipeline Layout which is destroyed automatically when it's dropped.
pub struct PipelineLayout {
    pub raw: vk::PipelineLayout,
    pub vk_dev: Arc<RenderDevice>,
}

impl PipelineLayout {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        descriptor_layouts: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, PipelineError> {
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            p_set_layouts: descriptor_layouts.as_ptr(),
            set_layout_count: descriptor_layouts.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .map_err(PipelineError::UnableToCreatePipelineLayout)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl VulkanDebug for PipelineLayout {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::PIPELINE_LAYOUT,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for PipelineLayout {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_pipeline_layout(self.raw, None);
        }
    }
}

use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{
    DescriptorSetLayout, RenderDevice, VulkanDebug, VulkanError,
};

/// An owned Vulkan pipeline which is automatically destroyed when dropped.
pub struct PipelineLayout {
    descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
    pipeline_layout: vk::PipelineLayout,
    render_device: Arc<RenderDevice>,
}

impl PipelineLayout {
    /// Create a new Pipeline Layout.
    pub fn new(
        render_device: Arc<RenderDevice>,
        descriptor_set_layouts: &[Arc<DescriptorSetLayout>],
    ) -> Result<Self, VulkanError> {
        let raw_descriptor_set_layouts: Vec<vk::DescriptorSetLayout> =
            descriptor_set_layouts
                .iter()
                .map(|descriptor_set_layout| unsafe {
                    // Safe because the pipeline layout retains references to
                    // the descriptor sets for the duration of its lifetime.
                    *descriptor_set_layout.raw()
                })
                .collect();
        let create_info = vk::PipelineLayoutCreateInfo {
            p_set_layouts: raw_descriptor_set_layouts.as_ptr(),
            set_layout_count: raw_descriptor_set_layouts.len() as u32,
            ..Default::default()
        };
        let pipeline_layout =
            unsafe { render_device.create_pipeline_layout(&create_info)? };
        Ok(Self {
            descriptor_set_layouts: descriptor_set_layouts.to_owned(),
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

    /// Get one of the descriptor sets used to create the pipeline layout.
    ///
    /// - index matches the indices of the descriptor set layout slice given
    ///   when the pipeline layout is constructed.
    pub fn descriptor_set_layout(
        &self,
        index: usize,
    ) -> &Arc<DescriptorSetLayout> {
        &self.descriptor_set_layouts[index]
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

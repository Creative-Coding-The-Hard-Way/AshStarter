use {
    super::raii_wrapper,
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

raii_wrapper!(
    PipelineLayout,
    PipelineLayoutCreateInfo,
    PIPELINE_LAYOUT,
    create_pipeline_layout,
    destroy_pipeline_layout
);

impl PipelineLayout {
    /// Create a new Vulkan pipeline layout.
    ///
    /// # Params
    ///
    /// * `render_device` - the Vulkan device used to create resources
    /// * `descriptor_set_layouts` - the descriptor set layouts used by the
    ///   pipeline
    /// * `push_constant_ranges` - the push constants used by the pipeline
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - any descriptor set layouts must live at least as long as the
    ///     pipeline layout
    ///   - the pipeline layout must be destroyed before exit
    pub unsafe fn new_with_layouts_and_ranges(
        render_device: Arc<RenderDevice>,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Result<Self, GraphicsError> {
        let create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: descriptor_set_layouts.len() as u32,
            p_set_layouts: if descriptor_set_layouts.is_empty() {
                std::ptr::null()
            } else {
                descriptor_set_layouts.as_ptr()
            },
            push_constant_range_count: push_constant_ranges.len() as u32,
            p_push_constant_ranges: if push_constant_ranges.is_empty() {
                std::ptr::null()
            } else {
                push_constant_ranges.as_ptr()
            },
            ..Default::default()
        };
        Self::new(render_device, &create_info)
    }
}

use {
    super::raii_wrapper,
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

raii_wrapper!(
    DescriptorSetLayout,
    DescriptorSetLayoutCreateInfo,
    DESCRIPTOR_SET_LAYOUT,
    create_descriptor_set_layout,
    destroy_descriptor_set_layout
);

impl DescriptorSetLayout {
    /// Create a new DescriptorSetLayout using the given bindings.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The DescriptorSetLayout must be dropped before the Vulkan device.
    ///   - The application must synchronize usage of this resource.
    pub unsafe fn new_with_bindings(
        render_device: Arc<RenderDevice>,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> Result<Self, GraphicsError> {
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: if bindings.is_empty() {
                std::ptr::null()
            } else {
                bindings.as_ptr()
            },
            ..Default::default()
        };
        DescriptorSetLayout::new(render_device, &create_info)
    }
}

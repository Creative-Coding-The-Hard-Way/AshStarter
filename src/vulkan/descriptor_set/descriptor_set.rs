use super::DescriptorSet;

use crate::vulkan::{errors::VulkanDebugError, VulkanDebug};

use ash::{version::DeviceV1_0, vk};

impl DescriptorSet {
    /// Write a buffer binding to this descripor set.
    ///
    /// # Unsafe
    ///
    /// - because the application must ensure the descriptor set is not in-use
    ///   when it modified by this function.
    pub unsafe fn bind_buffer(
        &self,
        binding: u32,
        buffer: &vk::Buffer,
        descriptor_type: vk::DescriptorType,
    ) {
        let descriptor_buffer_info = vk::DescriptorBufferInfo {
            buffer: *buffer,
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let write = vk::WriteDescriptorSet {
            dst_set: self.raw,
            dst_binding: binding,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type,
            p_image_info: std::ptr::null(),
            p_texel_buffer_view: std::ptr::null(),
            p_buffer_info: &descriptor_buffer_info,
            ..Default::default()
        };
        self.vk_dev
            .logical_device
            .update_descriptor_sets(&[write], &[]);
    }
}

impl VulkanDebug for DescriptorSet {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_SET,
            self.raw,
        )?;
        Ok(())
    }
}

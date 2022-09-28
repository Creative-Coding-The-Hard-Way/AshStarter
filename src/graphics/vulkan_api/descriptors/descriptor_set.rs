use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{
    DescriptorPool, DescriptorSetLayout, HostCoherentBuffer, RenderDevice,
    VulkanDebug, VulkanError,
};

/// An owned descriptor set.
/// Note: Descriptor Sets keep the DescriptorPool they were allocated from alive
/// until dropped.
pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
    _descriptor_pool: Arc<DescriptorPool>,
    render_device: Arc<RenderDevice>,
}

impl DescriptorSet {
    /// Allocate descriptor sets from a pool with a given descriptor set layout.
    pub fn allocate(
        render_device: &Arc<RenderDevice>,
        descriptor_pool: &Arc<DescriptorPool>,
        descriptor_set_layout: &DescriptorSetLayout,
        count: u32,
    ) -> Result<Vec<DescriptorSet>, VulkanError> {
        // Safe because the layout is only used when allocating and the raw
        // sets will each be paired with a clone of the pool arc.
        let raw_descriptor_sets = unsafe {
            descriptor_pool
                .allocate_descriptor_sets(count, *descriptor_set_layout.raw())?
        };
        Ok(raw_descriptor_sets
            .into_iter()
            .map(|raw_descriptor_set| Self {
                descriptor_set: raw_descriptor_set,
                _descriptor_pool: descriptor_pool.clone(),
                render_device: render_device.clone(),
            })
            .collect())
    }

    /// Get the raw Vulkan Descriptor Set handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Ownership is not transferred. The caller must ensure no reference to
    ///     the underlying handle outlives this object.
    pub unsafe fn raw(&self) -> &vk::DescriptorSet {
        &self.descriptor_set
    }

    /// Write a buffer to the descriptor set.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - device extensions are required if writing to a descriptor set while
    ///     it is bound
    ///   - the caller must ensure that the buffer lives at least as long as
    ///     the descriptor set while it's reference.d
    pub unsafe fn write_uniform_buffer<T>(
        &self,
        binding: u32,
        buffer: &HostCoherentBuffer<T>,
    ) {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: *buffer.raw(),
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        self.render_device.update_descriptor_sets(
            &[vk::WriteDescriptorSet {
                dst_set: self.descriptor_set,
                dst_binding: binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_image_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
                p_buffer_info: &buffer_info,
                ..Default::default()
            }],
            &[],
        )
    }
}

impl VulkanDebug for DescriptorSet {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::DESCRIPTOR_SET,
            self.descriptor_set,
        )
    }
}
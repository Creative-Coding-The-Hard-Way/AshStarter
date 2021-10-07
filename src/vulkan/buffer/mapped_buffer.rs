use crate::vulkan::RenderDevice;

use super::{Buffer, MappedBuffer};

use ash::version::DeviceV1_0;

impl<'data_type, T: 'data_type + Copy> MappedBuffer<'data_type, T> {
    /// Removed the memory mapping and return the original untyped buffer.
    pub fn unmap(self, vk_dev: &RenderDevice) -> Buffer {
        unsafe {
            vk_dev
                .logical_device
                .unmap_memory(self.buffer.allocation.memory);
        }
        self.buffer
    }
}

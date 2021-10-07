use crate::vulkan::RenderDevice;

use super::{Buffer, BufferError, MappedBuffer};

use ash::{version::DeviceV1_0, vk};

impl Buffer {
    /// Map this CPU buffer. The original unmapped data can be reclaimed using
    /// the `unmap` method.
    pub fn map<'a, T: 'a + Copy>(
        self,
        vk_dev: &RenderDevice,
    ) -> Result<MappedBuffer<'a, T>, BufferError> {
        let ptr = unsafe {
            vk_dev
                .logical_device
                .map_memory(
                    self.allocation.memory,
                    self.allocation.offset,
                    self.allocation.byte_size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(BufferError::UnableToMapDeviceMemory)?
        };
        let len = self.allocation.byte_size as usize / std::mem::size_of::<T>();

        let data =
            unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, len) };

        Ok(MappedBuffer { data, buffer: self })
    }
}

use crate::vulkan::RenderDevice;

use super::{Buffer, BufferError};

use ash::{version::DeviceV1_0, vk};

impl Buffer {
    /// Acquire a CPU-accessible pointer to the memory used by this buffer.
    ///
    /// # Errors
    ///
    /// * This will fail if the buffer was not created with the HOST_VISIBLE
    ///   property.
    /// * This will also fail if the buffer is already mapped.
    /// * This will fail if multiple buffers share the same memory -- like at
    ///   different offsets -- and all attempt to map the memory at the same
    ///   time.
    pub fn map(&mut self, vk_dev: &RenderDevice) -> Result<(), BufferError> {
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
        self.mapped_ptr = Some(ptr);
        Ok(())
    }

    /// Unmap the buffer's memory.
    pub fn unmap(&mut self, vk_dev: &RenderDevice) {
        unsafe {
            vk_dev.logical_device.unmap_memory(self.allocation.memory);
        }
        self.mapped_ptr = None;
    }

    /// Access the buffer's memory by treating it like a `&[Element]`.
    pub fn data<'element, Element: 'element + Copy>(
        &self,
    ) -> Result<&'element [Element], BufferError> {
        let ptr = self.mapped_ptr.ok_or(BufferError::NoMappedPointerFound)?;
        let elements =
            self.allocation.byte_size as usize / std::mem::size_of::<Element>();
        let data = unsafe {
            std::slice::from_raw_parts(ptr as *const Element, elements)
        };
        Ok(data)
    }

    /// Access the buffer's memory by treating it like a `&mut [Element]`.
    pub fn data_mut<'element, Element: 'element + Copy>(
        &self,
    ) -> Result<&'element mut [Element], BufferError> {
        let ptr = self.mapped_ptr.ok_or(BufferError::NoMappedPointerFound)?;
        let elements =
            self.allocation.byte_size as usize / std::mem::size_of::<Element>();
        let data = unsafe {
            std::slice::from_raw_parts_mut(ptr as *mut Element, elements)
        };
        Ok(data)
    }
}

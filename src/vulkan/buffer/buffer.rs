use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

use super::{Buffer, BufferError};
use crate::vulkan::{MemoryAllocator, RenderDevice, VulkanDebug};

impl Buffer {
    /// Create a new Vulkan buffer and bind it to device memory with at a
    /// requested size.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        size_in_bytes: u64,
    ) -> Result<Self, BufferError> {
        let create_info = vk::BufferCreateInfo {
            size: size_in_bytes,
            usage: buffer_usage_flags,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer_handle = unsafe {
            vk_dev
                .logical_device
                .create_buffer(&create_info, None)
                .map_err(|err| BufferError::UnableToCreateBuffer {
                    size: size_in_bytes,
                    usage: buffer_usage_flags,
                    source: err,
                })?
        };
        let allocation = unsafe {
            let buffer_memory_requirements = vk_dev
                .logical_device
                .get_buffer_memory_requirements(buffer_handle);
            vk_alloc.allocate_memory(
                buffer_memory_requirements,
                memory_property_flags,
            )?
        };
        unsafe {
            vk_dev
                .logical_device
                .bind_buffer_memory(
                    buffer_handle,
                    allocation.memory,
                    allocation.offset,
                )
                .map_err(BufferError::UnableToBindDeviceMemory)?;
        }

        Ok(Self {
            raw: buffer_handle,
            allocation,
            mapped_ptr: None,
            vk_alloc,
            vk_dev,
        })
    }

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
    pub fn map(&mut self) -> Result<(), BufferError> {
        let ptr = unsafe {
            self.vk_dev
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
    pub fn unmap(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .unmap_memory(self.allocation.memory);
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

impl Drop for Buffer {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev.logical_device.destroy_buffer(self.raw, None);
            self.vk_alloc
                .free(&self.allocation)
                .expect("unable to free the buffer's memory");
        }
    }
}

impl VulkanDebug for Buffer {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), crate::vulkan::vulkan_debug::VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::BUFFER,
            self.raw,
        )?;
        Ok(())
    }
}

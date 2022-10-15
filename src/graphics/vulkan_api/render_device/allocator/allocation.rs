use {
    crate::graphics::vulkan_api::{RenderDevice, VulkanError},
    ash::vk,
    std::ffi::c_void,
};

/// An allocated chunk of GPU memory.
pub struct Allocation {
    device_memory: vk::DeviceMemory,
    offset_in_bytes: vk::DeviceSize,
    size_in_bytes: vk::DeviceSize,
    memory_type_index: u32,
    cpu_mapped_ptr: Option<*mut c_void>,
}

// public api
impl Allocation {
    /// Get the size of the allocation in bytes.
    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes as usize
    }

    /// Get the device's memory type index.
    pub fn memory_type_index(&self) -> u32 {
        self.memory_type_index
    }

    /// Create a CPU-accessible pointer to the memory in this allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - only memmory accessible by the host can be mapped
    ///  - memory that is not HOST_COHERENT requires additional synchronization
    ///    after writes/reads
    ///  - the application is responsible for making a corresponding call to
    ///    unmap
    pub unsafe fn map(
        &mut self,
        device: &RenderDevice,
    ) -> Result<(), VulkanError> {
        let host_memory_ptr = device.map_memory(
            self.device_memory,
            self.offset_in_bytes,
            self.size_in_bytes,
        )?;
        let host_memory_ptr_with_offset = (host_memory_ptr as *mut u8)
            .add(self.offset_in_bytes as usize)
            as *mut c_void;
        self.cpu_mapped_ptr = Some(host_memory_ptr_with_offset);
        Ok(())
    }

    /// Unmap the cpu-accessible pointer to the memory in this allocation.
    pub fn unmap(&mut self, device: &RenderDevice) {
        if self.cpu_mapped_ptr.take().is_some() {
            // safe because this will only occur if the memory is already mapped
            unsafe { device.unmap_memory(self.device_memory) }
        }
    }

    /// Access the mapped device memory as a slice of T.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - a call to map() must be made by the application prior to calling this
    ///    function
    ///  - errors if the host-mapped pointer and offset are not correctly
    ///    aligned for the type T. Use #[repr(C, packed)] on types which will be
    ///    written into GPU buffers to have maximum control over memory layout.
    pub unsafe fn as_slice_mut<T>(&mut self) -> Result<&mut [T], VulkanError> {
        let mapped_ptr = self
            .cpu_mapped_ptr
            .ok_or(VulkanError::DeviceMemoryIsNotMapped)?;

        if (mapped_ptr as usize % std::mem::align_of::<T>()) != 0 {
            return Err(VulkanError::DeviceMemoryIsNotAlignedForType(
                std::any::type_name::<T>().to_owned(),
            ));
        }

        let number_of_elements =
            self.size_in_bytes as usize / std::mem::size_of::<T>();

        Ok(std::slice::from_raw_parts_mut(
            mapped_ptr as *mut T,
            number_of_elements,
        ))
    }

    /// Access the mapped device memory as a slice of T.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - a call to map() must be made by the application prior to calling this
    ///    function
    ///  - errors if the host-mapped pointer and offset are not correctly
    ///    aligned for the type T. Use #[repr(C, packed)] on types which will be
    ///    written into GPU buffers to have maximum control over memory layout.
    pub unsafe fn as_slice<T>(&self) -> Result<&[T], VulkanError> {
        let mapped_ptr = self
            .cpu_mapped_ptr
            .ok_or(VulkanError::DeviceMemoryIsNotMapped)?;

        if (mapped_ptr as usize % std::mem::align_of::<T>()) != 0 {
            return Err(VulkanError::DeviceMemoryIsNotAlignedForType(
                std::any::type_name::<T>().to_owned(),
            ));
        }

        let number_of_elements =
            self.size_in_bytes as usize / std::mem::size_of::<T>();

        Ok(std::slice::from_raw_parts(
            mapped_ptr as *const T,
            number_of_elements,
        ))
    }

    /// Flush host-side caches so changes are visible on the Vulkan device.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - it is invalid to call this function without previously calling map()
    pub unsafe fn flush_mapped_memory(
        &self,
        render_device: &RenderDevice,
    ) -> Result<(), VulkanError> {
        render_device.flush_mapped_memory_ranges(&[vk::MappedMemoryRange {
            memory: self.device_memory,
            offset: self.offset_in_bytes,
            size: self.size_in_bytes,
            ..Default::default()
        }])
    }
}

// internal api
impl Allocation {
    /// Create a new memory allocation with the given Vulkan memory handle.
    ///
    /// # Safety
    ///
    /// Unsafe because the memory object is *not* dropped automatically. The
    /// application is responsible for freeing the allocation when it is no
    /// longer in use.
    pub(super) unsafe fn new(
        memory: vk::DeviceMemory,
        offset_in_bytes: vk::DeviceSize,
        size_in_bytes: vk::DeviceSize,
        memory_type_index: u32,
    ) -> Self {
        Self {
            device_memory: memory,
            offset_in_bytes,
            size_in_bytes,
            memory_type_index,
            cpu_mapped_ptr: None,
        }
    }

    /// Get the allocation's offset from the front of the raw DeviceMemory
    /// pointer. Generally this is only used when binding the raw memory to
    /// some other resource.
    pub(in crate::graphics::vulkan_api::render_device) unsafe fn offset_in_bytes(
        &self,
    ) -> vk::DeviceSize {
        self.offset_in_bytes
    }

    /// Get the underlying device memory handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - ownership is not transferred, the allocation still owns the device
    ///     memory and will destroy it when freed
    pub(in crate::graphics::vulkan_api::render_device) unsafe fn device_memory(
        &self,
    ) -> vk::DeviceMemory {
        self.device_memory
    }
}

use {
    crate::graphics::vulkan_api::{
        Allocation, Buffer, RenderDevice, VulkanDebug, VulkanError,
    },
    ash::vk,
    std::{marker::PhantomData, sync::Arc},
};

/// A Vulkan Device buffer backed by a device-local memory allocation.
pub struct DeviceLocalBuffer<T> {
    element_count: usize,
    buffer: vk::Buffer,
    allocation: Allocation,
    render_device: Arc<RenderDevice>,
    _phantom_data: PhantomData<T>,
}

impl<T> DeviceLocalBuffer<T> {
    /// How many elements of type T are stored in this buffer.
    pub fn element_count(&self) -> usize {
        self.element_count
    }
}

impl<T> DeviceLocalBuffer<T>
where
    T: Copy,
{
    /// Create a new Device buffer that the host can read and write.
    ///
    /// len is the number of elements to be stored in the buffer.
    pub fn new(
        render_device: Arc<RenderDevice>,
        usage: vk::BufferUsageFlags,
        len: usize,
    ) -> Result<Self, VulkanError> {
        let size_in_bytes = len * std::mem::size_of::<T>();
        let create_info = vk::BufferCreateInfo {
            size: size_in_bytes as u64,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe { render_device.create_buffer(&create_info)? };
        let allocation = unsafe {
            let memory_requirements =
                render_device.get_buffer_memory_requirements(&buffer);
            render_device.allocate_memory(
                memory_requirements,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?
        };

        // safe because the buffer and allocation are held together in this
        // object
        unsafe { render_device.bind_buffer_memory(&buffer, &allocation)? };

        Ok(Self {
            element_count: len,
            buffer,
            allocation,
            render_device,
            _phantom_data: PhantomData::default(),
        })
    }
}

impl<T> Buffer for DeviceLocalBuffer<T> {
    unsafe fn raw(&self) -> vk::Buffer {
        self.buffer
    }

    fn size_in_bytes(&self) -> usize {
        self.allocation.size_in_bytes()
    }

    fn element_count(&self) -> usize {
        self.element_count
    }
}

impl<T> VulkanDebug for DeviceLocalBuffer<T> {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::BUFFER,
            self.buffer,
        );
    }
}

impl<T> Drop for DeviceLocalBuffer<T> {
    /// # Safety
    ///
    /// The application must ensure no Vulkan Device operations reference this
    /// buffer when it is dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_buffer(self.buffer);
            self.render_device
                .free_memory(&self.allocation)
                .expect("Unable to free the buffer's memory");
        }
    }
}

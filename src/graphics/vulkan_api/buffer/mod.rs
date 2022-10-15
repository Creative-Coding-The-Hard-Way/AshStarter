mod device_local_buffer;
mod host_coherent_buffer;

use {ash::vk, std::sync::Arc};

pub use self::{
    device_local_buffer::DeviceLocalBuffer,
    host_coherent_buffer::HostCoherentBuffer,
};

/// Functionality common to all buffer types.
pub trait Buffer {
    /// Get the raw Vulkan buffer handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - ownership is not transferred
    ///   - the caller is responsible for synchronizing access to the buffer
    unsafe fn raw(&self) -> vk::Buffer;

    /// Get the size of this buffer in bytes.
    fn size_in_bytes(&self) -> usize;

    /// Get the number of elements in this buffer.
    fn element_count(&self) -> usize;
}

impl Buffer for Arc<dyn Buffer> {
    unsafe fn raw(&self) -> vk::Buffer {
        self.as_ref().raw()
    }

    fn size_in_bytes(&self) -> usize {
        self.as_ref().size_in_bytes()
    }

    fn element_count(&self) -> usize {
        self.as_ref().element_count()
    }
}

impl<T> Buffer for Arc<T>
where
    T: Buffer,
{
    unsafe fn raw(&self) -> vk::Buffer {
        self.as_ref().raw()
    }

    fn size_in_bytes(&self) -> usize {
        self.as_ref().size_in_bytes()
    }

    fn element_count(&self) -> usize {
        self.as_ref().element_count()
    }
}

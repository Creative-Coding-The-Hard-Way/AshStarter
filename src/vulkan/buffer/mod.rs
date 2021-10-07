mod buffer;
mod mapped_buffer;

use crate::vulkan::Allocation;

use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Unable to map device memory")]
    UnableToMapDeviceMemory(#[source] vk::Result),
}

/// A Vulkan buffer and it's associated device memory.
pub struct Buffer {
    /// The underlying Vulkan buffer type
    pub raw: vk::Buffer,

    /// The actual memory alloctaion for this buffer
    pub allocation: Allocation,
}

/// A CPU readable/writable buffer.
///
/// # Note
///
/// A normal buffer can be transformed into a MappedBuffer if and only if it
/// was created with the HOST_VISIBLE property flags. No memory flushes are
/// required if the original buffer was also created with the HOST_COHERENT
/// flag. See the vulkan spec:
/// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkMemoryPropertyFlagBits.html
pub struct MappedBuffer<'data_type, T: 'data_type + Copy> {
    /// The mapped buffer data. Read and write this data like a normal slice.
    pub data: &'data_type mut [T],

    /// The underlying unmapped buffer.
    pub buffer: Buffer,
}

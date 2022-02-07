use ash::vk;

/// A single allocated piece of device memory.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub byte_size: vk::DeviceSize,
    pub memory_type_index: u32,
}

impl Allocation {
    /// An empty allocation with null pointers and resource references.
    pub fn null() -> Allocation {
        Self {
            memory: vk::DeviceMemory::null(),
            offset: 0,
            byte_size: 0,
            memory_type_index: 0,
        }
    }
}

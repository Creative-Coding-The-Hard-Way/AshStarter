use super::Allocation;

use ash::vk;

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

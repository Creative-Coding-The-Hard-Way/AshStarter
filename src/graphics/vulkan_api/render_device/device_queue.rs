use ash::vk;

/// The Vulkan queue and associated indices.
#[derive(Debug, Clone, Copy)]
pub struct DeviceQueue {
    /// The Vulkan queue
    queue: vk::Queue,

    /// The family index for this queue
    family_index: u32,

    /// The index of the queue within the family
    queue_index: u32,
}

impl DeviceQueue {
    /// Create a new queue instance from the raw vulkan resource.
    pub fn from_raw(
        queue: vk::Queue,
        family_index: u32,
        queue_index: u32,
    ) -> Self {
        Self {
            queue,
            family_index,
            queue_index,
        }
    }

    /// Returns true if the queues refer to the same underlying resource.
    pub fn is_same(&self, device_queue: &Self) -> bool {
        self.family_index == device_queue.family_index
            && self.queue_index == device_queue.queue_index
    }
}

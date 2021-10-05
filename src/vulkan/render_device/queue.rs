use super::GpuQueue;

use ash::vk;

impl GpuQueue {
    /// Build a queue wrapper from the raw queue handle.
    pub fn from_raw(queue: vk::Queue, family_id: u32, index: u32) -> Self {
        Self {
            queue,
            family_id,
            index,
        }
    }

    /// Returns true if this instance and another represent the same device
    /// queue
    pub fn is_same(&self, queue: &GpuQueue) -> bool {
        self.family_id == queue.family_id && self.index == queue.index
    }
}

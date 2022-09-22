use std::sync::Arc;

use crate::graphics::vulkan_api::{RenderDevice, Semaphore, VulkanError};

/// A semaphore pool maintains a collection of binary semaphores which are
/// available for re-use.
/// Unused semaphores are automatically destroyed when the pool is dropped.
pub struct SemaphorePool {
    recycled_semaphores: Vec<Semaphore>,
    pub render_device: Arc<RenderDevice>,
}

impl SemaphorePool {
    /// Create a new semaphore pool.
    pub fn new(render_device: Arc<RenderDevice>) -> Self {
        Self {
            recycled_semaphores: vec![],
            render_device,
        }
    }

    /// Get a semaphore from the pool, or create a new one if none are
    /// available.
    pub fn get_semaphore(&mut self) -> Result<Semaphore, VulkanError> {
        if let Some(recycled) = self.recycled_semaphores.pop() {
            Ok(recycled)
        } else {
            Semaphore::new(self.render_device.clone())
        }
    }

    /// Return a semaphore to the pool for future use.
    pub fn return_semaphore(&mut self, semaphore: Semaphore) {
        self.recycled_semaphores.push(semaphore);
    }
}

use ::std::sync::Arc;

use crate::vulkan::{
    sync::{Semaphore, SemaphoreError},
    RenderDevice,
};

/// A semaphore pool maintains a collection of binary semaphores which are
/// available for re-use.
/// Unused semaphores are automatically destroyed when the pool is dropped.
pub struct SemaphorePool {
    recycled_semaphores: Vec<Semaphore>,
    pub vk_dev: Arc<RenderDevice>,
}

impl SemaphorePool {
    /// Create a new semaphore pool.
    pub fn new(vk_dev: Arc<RenderDevice>) -> Self {
        Self {
            recycled_semaphores: vec![],
            vk_dev,
        }
    }

    /// Get a semaphore from the pool, or create a new one if none are
    /// available.
    ///
    pub fn get_semaphore(&mut self) -> Result<Semaphore, SemaphoreError> {
        if let Some(recycled) = self.recycled_semaphores.pop() {
            Ok(recycled)
        } else {
            Semaphore::new(self.vk_dev.clone())
        }
    }

    /// Return a semaphore to the pool for future use.
    pub fn return_semaphore(&mut self, semaphore: Semaphore) {
        self.recycled_semaphores.push(semaphore);
    }
}

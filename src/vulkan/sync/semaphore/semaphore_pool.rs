use ::std::sync::Arc;

use super::{Semaphore, SemaphoreError, SemaphorePool};
use crate::vulkan::RenderDevice;

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

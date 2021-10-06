use crate::vulkan::RenderDevice;

use ash::{version::DeviceV1_0, vk};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemaphorePoolError {
    #[error("Unable to create a new semaphore")]
    UnableToCreateNewSemaphore(#[source] vk::Result),
}

/// A semaphore pool maintains a collection of binary semaphores which are
/// available for re-use. The application is responsible for destroying the
/// pool prior to exit.
pub struct SemaphorePool {
    recycled_semaphores: Vec<vk::Semaphore>,
}

impl SemaphorePool {
    /// Create a new semaphore pool.
    pub fn new() -> Self {
        Self {
            recycled_semaphores: vec![],
        }
    }

    /// Get a semaphore from the pool, or create a new one if none are
    /// available.
    ///
    /// The caller is responsible for destroying the semaphore or returning
    /// the semaphore to this pool to be recycled.
    pub fn get_semaphore(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> Result<vk::Semaphore, SemaphorePoolError> {
        if let Some(recycled) = self.recycled_semaphores.pop() {
            Ok(recycled)
        } else {
            let create_info = vk::SemaphoreCreateInfo {
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_semaphore(&create_info, None)
                    .map_err(SemaphorePoolError::UnableToCreateNewSemaphore)
            }
        }
    }

    /// Return a semaphore to the pool for future use.
    ///
    /// This function is a no-op if the semaphore is null.
    pub fn return_semaphore(&mut self, semaphore: vk::Semaphore) {
        if semaphore != vk::Semaphore::null() {
            self.recycled_semaphores.push(semaphore);
        }
    }

    /// Destroy any remaining returned semaphores.
    pub fn destroy(&mut self, vk_dev: &RenderDevice) {
        for semaphore in self.recycled_semaphores.drain(..) {
            unsafe {
                vk_dev.logical_device.destroy_semaphore(semaphore, None);
            }
        }
    }
}

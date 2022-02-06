mod semaphore;
mod semaphore_pool;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

use crate::vulkan::RenderDevice;

#[derive(Debug, Error)]
pub enum SemaphoreError {
    #[error("Unable to create a new semaphore")]
    UnableToCreateSemaphore(#[source] vk::Result),
}

/// An owned semaphore which is automatically destroyed when it is dropped.
pub struct Semaphore {
    pub raw: vk::Semaphore,
    pub vk_dev: Arc<RenderDevice>,
}

/// A semaphore pool maintains a collection of binary semaphores which are
/// available for re-use.
/// Unused semaphores are automatically destroyed when the pool is dropped.
pub struct SemaphorePool {
    recycled_semaphores: Vec<Semaphore>,
    pub vk_dev: Arc<RenderDevice>,
}

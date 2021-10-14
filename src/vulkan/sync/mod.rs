//! Vulkan synchronization privite wrappers.

pub(super) mod fence;
pub(super) mod semaphore;

pub use self::{
    fence::Fence,
    semaphore::{Semaphore, SemaphorePool},
};

mod ffi;
mod instance;
mod render_device;
mod semaphore_pool;
mod window_surface;

pub use self::{
    instance::Instance,
    render_device::{GpuQueue, RenderDevice, VulkanDebugName},
    semaphore_pool::SemaphorePool,
    window_surface::WindowSurface,
};

pub mod errors {
    use thiserror::Error;

    pub use super::{
        instance::InstanceError,
        render_device::{
            PhysicalDeviceError, QueueSelectionError, RenderDeviceError,
            SwapchainError,
        },
        semaphore_pool::SemaphorePoolError,
        window_surface::WindowSurfaceError,
    };

    #[derive(Debug, Error)]
    pub enum VulkanError {
        #[error(transparent)]
        InstanceError(#[from] InstanceError),

        #[error(transparent)]
        PhysicalDeviceError(#[from] PhysicalDeviceError),

        #[error(transparent)]
        QueueSelectionError(#[from] QueueSelectionError),

        #[error(transparent)]
        RenderDeviceError(#[from] RenderDeviceError),

        #[error(transparent)]
        SwapchainError(#[from] SwapchainError),

        #[error(transparent)]
        SemaphorePoolError(#[from] SemaphorePoolError),

        #[error(transparent)]
        WindowSurfaceError(#[from] WindowSurfaceError),
    }
}

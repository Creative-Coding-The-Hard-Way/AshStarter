mod buffer;
mod device_allocator;
mod ffi;
mod instance;
mod render_device;
mod semaphore_pool;
mod window_surface;

pub use self::{
    buffer::{Buffer, MappedBuffer},
    device_allocator::{
        create_default_allocator, Allocation, BufferAllocator, DeviceAllocator,
    },
    instance::Instance,
    render_device::{GpuQueue, RenderDevice, RenderPassArgs},
    semaphore_pool::SemaphorePool,
    window_surface::WindowSurface,
};

pub mod errors {
    use thiserror::Error;

    pub use super::{
        buffer::BufferError,
        device_allocator::DeviceAllocatorError,
        instance::InstanceError,
        render_device::{
            PhysicalDeviceError, QueueSelectionError, RenderDeviceError,
            ShaderModuleError, SwapchainError,
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

        #[error(transparent)]
        DeviceAllocatorError(#[from] DeviceAllocatorError),

        #[error(transparent)]
        BufferError(#[from] BufferError),

        #[error(transparent)]
        ShaderModuleError(#[from] ShaderModuleError),
    }
}

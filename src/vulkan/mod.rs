mod buffer;
mod command_buffer;
mod device_allocator;
mod ffi;
mod framebuffer;
mod instance;
mod render_device;
mod render_pass;
mod shader_module;
mod vulkan_debug;
mod window_surface;

pub mod sync;

pub use self::{
    buffer::Buffer,
    command_buffer::{CommandBuffer, CommandPool},
    device_allocator::{
        create_default_allocator, Allocation, ComposableAllocator,
        LockedMemoryAllocator, MemoryAllocator, PassthroughAllocator,
    },
    framebuffer::Framebuffer,
    instance::Instance,
    render_device::{GpuQueue, RenderDevice},
    render_pass::RenderPass,
    shader_module::ShaderModule,
    vulkan_debug::VulkanDebug,
    window_surface::WindowSurface,
};

pub mod errors {
    use thiserror::Error;

    pub use super::{
        buffer::BufferError,
        command_buffer::CommandBufferError,
        device_allocator::AllocatorError,
        framebuffer::FramebufferError,
        instance::InstanceError,
        render_device::{
            PhysicalDeviceError, QueueSelectionError, RenderDeviceError,
            SwapchainError,
        },
        render_pass::RenderPassError,
        shader_module::ShaderModuleError,
        sync::fence::FenceError,
        sync::semaphore::SemaphoreError,
        vulkan_debug::VulkanDebugError,
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
        SemaphorePoolError(#[from] SemaphoreError),

        #[error(transparent)]
        WindowSurfaceError(#[from] WindowSurfaceError),

        #[error(transparent)]
        AllocatorError(#[from] AllocatorError),

        #[error(transparent)]
        BufferError(#[from] BufferError),

        #[error(transparent)]
        ShaderModuleError(#[from] ShaderModuleError),

        #[error(transparent)]
        VulkanDebugError(#[from] VulkanDebugError),

        #[error(transparent)]
        FramebufferError(#[from] FramebufferError),

        #[error(transparent)]
        FenceError(#[from] FenceError),

        #[error(transparent)]
        CommandBufferError(#[from] CommandBufferError),

        #[error(transparent)]
        RenderPassError(#[from] RenderPassError),
    }
}

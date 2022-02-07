mod buffer;
mod command_buffer;
mod descriptor_set;
mod device_allocator;
mod ffi;
mod framebuffer;
mod image;
mod instance;
mod pipeline;
mod render_device;
mod render_pass;
mod vulkan_debug;
mod window_surface;

pub mod sync;

pub use self::{
    buffer::{Buffer, GpuVec},
    command_buffer::{CommandBuffer, CommandPool, OneTimeSubmitCommandPool},
    descriptor_set::{DescriptorPool, DescriptorSet, DescriptorSetLayout},
    device_allocator::{
        create_default_allocator, Allocation, ComposableAllocator,
        LockedMemoryAllocator, MemoryAllocator, PassthroughAllocator,
    },
    framebuffer::Framebuffer,
    image::{Image, ImageView},
    instance::Instance,
    pipeline::{Pipeline, PipelineLayout, ShaderModule},
    render_device::{GpuQueue, RenderDevice},
    render_pass::RenderPass,
    vulkan_debug::VulkanDebug,
    window_surface::WindowSurface,
};

pub mod errors {
    use thiserror::Error;

    pub use super::{
        buffer::BufferError,
        command_buffer::CommandBufferError,
        descriptor_set::DescriptorSetError,
        device_allocator::AllocatorError,
        framebuffer::FramebufferError,
        image::ImageError,
        instance::InstanceError,
        pipeline::PipelineError,
        render_device::{
            PhysicalDeviceError, QueueSelectionError, RenderDeviceError,
            SwapchainError,
        },
        render_pass::RenderPassError,
        sync::{FenceError, SemaphoreError},
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
        PipelineError(#[from] PipelineError),

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

        #[error(transparent)]
        DescriptorSetError(#[from] DescriptorSetError),

        #[error(transparent)]
        ImageError(#[from] ImageError),
    }
}

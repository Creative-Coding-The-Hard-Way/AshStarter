mod command_buffer;
mod command_pool;
mod one_time_submit_command_pool;

use crate::vulkan::{GpuQueue, RenderDevice};

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum CommandBufferError {
    #[error("Unable to create a new command buffer pool")]
    UnableToCreateCommandPool(#[source] vk::Result),

    #[error("Unable to allocate a command buffer from the command pool")]
    UnableToAllocateBuffer(#[source] vk::Result),

    #[error("Unable to reset the command pool")]
    UnableToResetPool(#[source] vk::Result),
}

/// A Vulkan CommandBuffer wrapper which automatically frees the buffer when
/// its dropped.
pub struct CommandBuffer {
    /// The raw vulkan command buffer handle.
    pub raw: vk::CommandBuffer,

    /// The CommandPool which was used to allocate this buffer.
    pub pool: Arc<CommandPool>,

    /// The vulkan device which created this command buffer.
    pub vk_dev: Arc<RenderDevice>,
}

/// A Vulkan CommandPool wrapper which automatically destroys the pool when it's
/// dropped.
pub struct CommandPool {
    /// The raw vulkan command pool handle
    pub raw: vk::CommandPool,

    /// The vulkan device which created the command pool
    pub vk_dev: Arc<RenderDevice>,
}

/// A command pool + command buffer combo which provides a convenient method
/// for synchronously submitting commands to a queue.
pub struct OneTimeSubmitCommandPool {
    pool: Arc<CommandPool>,
    cmd: CommandBuffer,
    queue: GpuQueue,

    /// The vulkan device used to create this
    pub vk_dev: Arc<RenderDevice>,
}

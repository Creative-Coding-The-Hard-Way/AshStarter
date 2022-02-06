use ::{ash::vk, thiserror::Error};

#[derive(Debug, Error)]
pub enum CommandBufferError {
    #[error("Unable to create a new command buffer pool")]
    UnableToCreateCommandPool(#[source] vk::Result),

    #[error("Unable to allocate a command buffer from the command pool")]
    UnableToAllocateBuffer(#[source] vk::Result),

    #[error("Unable to reset the command pool")]
    UnableToResetPool(#[source] vk::Result),
}

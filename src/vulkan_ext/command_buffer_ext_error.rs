use ::{ash::vk, thiserror::Error};

#[derive(Debug, Error)]
pub enum CommandBufferExtError {
    #[error("Unable to begin the command buffer")]
    UnableToBeginCommandBuffer(#[source] vk::Result),

    #[error("Unable to end the command buffer")]
    UnableToEndCommandBuffer(#[source] vk::Result),
}

pub type CommandResult<T> = Result<T, CommandBufferExtError>;

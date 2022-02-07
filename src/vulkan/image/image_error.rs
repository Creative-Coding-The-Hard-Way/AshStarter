use ::{ash::vk, thiserror::Error};

use crate::vulkan::errors::AllocatorError;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Unable to create a new image")]
    UnableToCreateImage(#[source] vk::Result),

    #[error("Unable to allocate memory for a new image")]
    UnableToAllocateImageMemory(#[from] AllocatorError),

    #[error("Unable to bind memory to the new image")]
    UnableToBindImageMemory(#[source] vk::Result),

    #[error("Unable to create Image View")]
    UnableToCreateView(#[source] vk::Result),
}

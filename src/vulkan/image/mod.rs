mod image;
mod image_view;

use crate::vulkan::{
    errors::AllocatorError, Allocation, MemoryAllocator, RenderDevice,
};

use ::{ash::vk, std::sync::Arc, thiserror::Error};

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

/// A owned Vulkan image handle which is automatically destroyed when it is
/// dropped.
pub struct Image {
    /// The Vulkan image handle.
    pub raw: vk::Image,

    /// A region of allocated memory which is bound to the image.
    pub allocation: Allocation,

    /// The memory allocater used to create the image.
    pub vk_alloc: Arc<dyn MemoryAllocator>,

    /// The render device used to create the image.
    pub vk_dev: Arc<RenderDevice>,
}

/// An owned Vulkan image view which is automatically destroyed when it is
/// dropped.
pub struct ImageView {
    /// The raw image view handle
    pub raw: vk::ImageView,

    /// The image associated with this view.
    pub image: Arc<Image>,

    /// The render device used to create the image view.
    pub vk_dev: Arc<RenderDevice>,
}

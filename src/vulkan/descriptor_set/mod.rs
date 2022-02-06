mod descriptor_pool;
mod descriptor_set;
mod descriptor_set_layout;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

use crate::vulkan::RenderDevice;

#[derive(Debug, Error)]
pub enum DescriptorSetError {
    #[error("Unable to create the descriptor set layout")]
    UnableToCreateLayout(#[source] vk::Result),

    #[error("Unable to create the descriptor pool")]
    UnableToCreatePool(#[source] vk::Result),

    #[error("Unable to allocate descriptors from the pool")]
    UnableToAllocateDescriptors(#[source] vk::Result),
}

/// A Vulkan descriptor set wrapper.
pub struct DescriptorSet {
    /// the raw Vulkan descriptor set handle.
    pub raw: vk::DescriptorSet,

    /// The device used to create the descriptor set.
    pub vk_dev: Arc<RenderDevice>,
}

/// An owned Descriptor Pool which is automatically destroyed when dropped.
pub struct DescriptorPool {
    /// The raw vulkan descriptor pool handle.
    pub raw: vk::DescriptorPool,

    /// The device used to create the pool.
    pub vk_dev: Arc<RenderDevice>,
}

/// An owned Descriptor Set Layout which is automatically destroyed when
/// dropped.
pub struct DescriptorSetLayout {
    /// The raw vulkan Descriptor Set Layout handle
    pub raw: vk::DescriptorSetLayout,

    /// The device used to create the layout
    pub vk_dev: Arc<RenderDevice>,
}

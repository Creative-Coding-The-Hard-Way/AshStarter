use {ash::vk, ccthw_ash_instance::InstanceError, thiserror::Error};

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("No suitable physical device could be found!")]
    NoSuitablePhysicalDevice,

    #[error(transparent)]
    RuntimeError(#[from] anyhow::Error),

    #[error(transparent)]
    InstanceError(#[from] InstanceError),

    #[error(transparent)]
    VulkanError(#[from] vk::Result),
}

use {crate::graphics::vulkan_api::VulkanError, thiserror::Error};

#[derive(Debug, Error)]
pub enum ApplicationError {
    /// Indicates that the application was unable to initialize the GLFW
    /// library.
    #[error(transparent)]
    GlfwInitError(#[from] glfw::InitError),

    #[error("Vulkan is not supported by GLFW on this system")]
    GlfwVulkanNotSupported,

    #[error("Unable to create the GLFW window")]
    UnableToCreateGLFWWindow,

    #[error("GLFW is unable to find a primary monitor")]
    NoPrimaryMonitor,

    #[error("GLFW cannot determine the monitor's primary video mode")]
    NoPrimaryVideoMode,

    #[error(transparent)]
    VulkanError(#[from] VulkanError),

    #[error("Unable to get the required Vulkan extensions using GLFW")]
    UnableToGetGLFWInstanceExtensions,

    #[error("Unable to create a Vulkan KHR Surface {:?}", .0)]
    UnableToCreateSurface(#[source] ash::vk::Result),
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    GlfwInitError(#[from] glfw::InitError),

    #[error("Vulkan is not supported by GLFW on this system")]
    GlfwVulkanNotSupported,

    #[error("Failed to create the GLFW window.")]
    CreateGLFWWindowFailed,

    #[error("GLFW is unable to find a primary monitor")]
    NoPrimaryMonitor,

    #[error("GLFW cannot determine the monitor's primary video mode")]
    NoPrimaryVideoMode,
}

mod ffi;
mod instance;
mod render_device;
mod window_surface;

pub use instance::Instance;
pub use render_device::{GpuQueue, RenderDevice, VulkanDebugName};
pub use window_surface::WindowSurface;

pub mod errors {
    pub use super::instance::InstanceError;
    pub use super::render_device::{
        PhysicalDeviceError, QueueSelectionError, RenderDeviceError,
        SwapchainError,
    };
    pub use super::window_surface::WindowSurfaceError;
}

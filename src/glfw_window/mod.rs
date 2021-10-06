use crate::vulkan::{
    errors::{InstanceError, RenderDeviceError, SwapchainError},
    Instance, RenderDevice, WindowSurface,
};

use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, vk::Handle};
use std::sync::mpsc::Receiver;
use thiserror::Error;

/// Window Errors represent things which can go wrong while creating and
/// manipulating GLFW windows.
#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Failed to create the GLFW window")]
    UnableToInitGLFW(#[from] glfw::InitError),

    #[error("Vulkan is not supported on this device")]
    VulkanNotSupported,

    #[error("The GLFW Window could not be created")]
    WindowCreateFailed,

    #[error("The Window's event reciever has already been taken")]
    EventReceiverLost,

    #[error("There is no primary monitor available to this GLFW instance")]
    NoPrimaryMonitor,

    #[error("There is no video mode associated with the primary monitor")]
    PrimaryVideoModeMissing,

    #[error("GLFW is unable to determine the required vulkan extensions for this platform")]
    RequiredExtensionsUnavailable,

    #[error("Unexpected instance error")]
    UnexpectedInstanceError(#[from] InstanceError),

    #[error("Unable to create the Vulkan surface")]
    UnableToCreateSurface(#[source] vk::Result),

    #[error("Unable to create the Vulkan render device")]
    UnexpectedRenderDeviceError(#[from] RenderDeviceError),

    #[error("Unexpected swapchain error")]
    UnexpectedSwapchainError(#[from] SwapchainError),
}

/// GLFW uses a Receiver for accepting window events. This type alias is more
/// convenient to write/read than the full name.
pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

/// All of the GLFW resources which are required for managing a single-windowed
/// GLFW application.
pub struct GlfwWindow {
    /// The glfw library instance
    pub glfw: glfw::Glfw,

    /// The glfw window
    pub window: glfw::Window,

    /// The event receiver which is typically consumed by the application's
    /// main loop.
    event_receiver: Option<EventReceiver>,

    /// The window's position before being put into fullscreen mode.
    window_pos: (i32, i32),

    /// The window's size before being put into fullscreen mode.
    window_size: (i32, i32),
}

impl GlfwWindow {
    /// Initialize the GLFW library and create a new window.
    pub fn new(window_title: &str) -> Result<Self, WindowError> {
        // Initialize the GLFW library
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        if !glfw.vulkan_supported() {
            return Err(WindowError::VulkanNotSupported);
        }

        // Tell GLFW not to bother setting up the OpenGL API
        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));

        // Create a windowed application. Fullscreen can always be toggled
        // later.
        let (window, event_receiver) = glfw
            .create_window(1366, 768, window_title, glfw::WindowMode::Windowed)
            .ok_or(WindowError::WindowCreateFailed)?;

        let window_pos = window.get_pos();
        let window_size = window.get_size();

        Ok(Self {
            glfw,
            window,
            event_receiver: Some(event_receiver),
            window_pos,
            window_size,
        })
    }

    /// Take ownership of this window's event reciever. This receiver can then
    /// be used to flush window events.
    pub fn take_event_receiver(
        &mut self,
    ) -> Result<EventReceiver, WindowError> {
        self.event_receiver
            .take()
            .ok_or(WindowError::EventReceiverLost)
    }

    /// Poll GLFW for window events and flush out into an iterator.
    pub fn flush_window_events<'events>(
        &mut self,
        event_receiver: &'events EventReceiver,
    ) -> glfw::FlushedMessages<'events, (f64, glfw::WindowEvent)> {
        self.glfw.poll_events();
        glfw::flush_messages(event_receiver)
    }

    /// Toggle the window in and out of fullcreen mode on the primary monitor.
    pub fn toggle_fullscreen(&mut self) -> Result<(), WindowError> {
        use glfw::WindowMode;
        let is_fullscreen = self.window.with_window_mode(|mode| match mode {
            WindowMode::Windowed => false,
            WindowMode::FullScreen(_) => true,
        });

        if is_fullscreen {
            // Switch to windowed mode.
            let (x, y) = self.window_pos;
            let (w, h) = self.window_size;
            self.window.set_monitor(
                WindowMode::Windowed,
                x,
                y,
                w as u32,
                h as u32,
                None,
            );
        } else {
            // Switch to fullscreen mode.
            // Record the size and position of the non-fullscreen window
            // before switching modes.
            self.window_size = self.window.get_size();
            self.window_pos = self.window.get_pos();
            let window = &mut self.window;
            self.glfw.with_primary_monitor_mut(
                |_, monitor_opt| -> Result<(), WindowError> {
                    let monitor =
                        monitor_opt.ok_or(WindowError::NoPrimaryMonitor)?;
                    let video_mode = monitor
                        .get_video_mode()
                        .ok_or(WindowError::PrimaryVideoModeMissing)?;
                    window.set_monitor(
                        WindowMode::FullScreen(monitor),
                        0,
                        0,
                        video_mode.width,
                        video_mode.height,
                        Some(video_mode.refresh_rate),
                    );
                    Ok(())
                },
            )?;
        }
        Ok(())
    }

    /// Create the Vulkan instance and surface for the current window.
    pub fn create_vulkan_device(&self) -> Result<RenderDevice, WindowError> {
        let required_extensions = self
            .glfw
            .get_required_instance_extensions()
            .ok_or(WindowError::RequiredExtensionsUnavailable)?;
        let instance = Instance::new(&required_extensions)?;

        let mut surface_handle: u64 = 0;
        let result = vk::Result::from_raw(self.window.create_window_surface(
            instance.ash.handle().as_raw() as usize,
            std::ptr::null(),
            &mut surface_handle,
        ) as i32);
        if result != vk::Result::SUCCESS {
            return Err(WindowError::UnableToCreateSurface(result));
        }

        let window_surface = WindowSurface::new(
            vk::SurfaceKHR::from_raw(surface_handle),
            Surface::new(&instance.entry, &instance.ash),
        );

        let mut device = RenderDevice::new(instance, window_surface)
            .map_err(WindowError::UnexpectedRenderDeviceError)?;

        let (w, h) = self.window.get_framebuffer_size();
        device.rebuild_swapchain((w as u32, h as u32))?;

        Ok(device)
    }
}

use {
    crate::application::ApplicationError,
    anyhow::Result,
    glfw::{ClientApiHint, WindowEvent, WindowHint, WindowMode},
    std::sync::mpsc::Receiver,
};

/// All resources required for running a single-windowed GLFW application which
/// renders graphics using Vulkan.
///
/// GlfwWindow derefs as a raw GLFW window handle so application state can
/// configure the window however is convenient.
pub struct GlfwWindow {
    window_pos: (i32, i32),
    window_size: (i32, i32),
    window_handle: glfw::Window,

    /// The receiver for the Window's events.
    pub(super) event_receiver: Option<Receiver<(f64, WindowEvent)>>,

    /// The GLFW library instance.
    pub(super) glfw: glfw::Glfw,
}

impl std::ops::Deref for GlfwWindow {
    type Target = glfw::Window;

    fn deref(&self) -> &Self::Target {
        &self.window_handle
    }
}

impl std::ops::DerefMut for GlfwWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window_handle
    }
}

impl GlfwWindow {
    /// Create a new GLFW window.
    /// Window hints and configuration can be done after the fact by using the
    /// underlying window handle.
    pub fn new(
        window_title: impl AsRef<str>,
    ) -> Result<Self, ApplicationError> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        if !glfw.vulkan_supported() {
            return Err(ApplicationError::GlfwVulkanNotSupported);
        }

        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::ScaleToMonitor(true));

        let (window_handle, event_receiver) = glfw
            .create_window(
                1366,
                768,
                window_title.as_ref(),
                WindowMode::Windowed,
            )
            .ok_or(ApplicationError::CreateGLFWWindowFailed)?;

        Ok(Self {
            window_pos: window_handle.get_pos(),
            window_size: window_handle.get_size(),
            event_receiver: Some(event_receiver),
            window_handle,
            glfw,
        })
    }

    pub fn toggle_fullscreen(&mut self) -> Result<(), ApplicationError> {
        let is_fullscreen =
            self.window_handle.with_window_mode(|mode| match mode {
                WindowMode::Windowed => false,
                WindowMode::FullScreen(_) => true,
            });

        if is_fullscreen {
            // Switch to windowed mode.
            let (x, y) = self.window_pos;
            let (w, h) = self.window_size;
            self.window_handle.set_monitor(
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
            self.window_size = self.window_handle.get_size();
            self.window_pos = self.window_handle.get_pos();
            let window = &mut self.window_handle;
            self.glfw.with_primary_monitor_mut(
                |_, monitor_opt| -> Result<(), ApplicationError> {
                    let monitor = monitor_opt
                        .ok_or(ApplicationError::NoPrimaryMonitor)?;
                    let video_mode = monitor
                        .get_video_mode()
                        .ok_or(ApplicationError::NoPrimaryVideoMode)?;
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
}

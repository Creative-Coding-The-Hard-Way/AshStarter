use {
    crate::graphics::vulkan_api::RenderDevice,
    anyhow::{bail, Context, Result},
    ccthw_ash_instance::{PhysicalDeviceFeatures, VulkanInstance},
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

impl GlfwWindow {
    /// Create a new GLFW window.
    ///
    /// The window starts in "windowed" mode and can be toggled into fullscreen
    /// or resized by the application.
    ///
    /// # Params
    ///
    /// * `window_title` - The title shown on the window's top bar.
    pub fn new(window_title: impl AsRef<str>) -> Result<Self> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        if !glfw.vulkan_supported() {
            bail!("Vulkan isn't supported by glfw on this platform!");
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
            .context("Creating the GLFW Window failed!")?;

        Ok(Self {
            window_pos: window_handle.get_pos(),
            window_size: window_handle.get_size(),
            event_receiver: Some(event_receiver),
            window_handle,
            glfw,
        })
    }

    /// Toggle application fullscreen.
    ///
    /// If the window is currently windowed then swap to fullscreen using
    /// whatever the primary monitor advertises as the primary video mode.
    ///
    /// If the window is currently fullscreen, then swap to windowed and
    /// restore the window's previous size and location.
    pub fn toggle_fullscreen(&mut self) -> Result<()> {
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
                |_, monitor_opt| -> Result<()> {
                    let monitor = monitor_opt
                        .context("Unable to determine the primary monitor!")?;
                    let video_mode = monitor
                        .get_video_mode()
                        .context("Unable to get a primary video mode for the primary monitor!")?;
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

    /// Create a render device for the application.
    ///
    /// # Params
    ///
    /// * `instance_extensions` - Any extensions to enable when creating the
    ///   instance. Extensions for the swapchain on the current platform are
    ///   added automatically and do not need to be provided.
    /// * `instance_layers` - Any additional layers to provide. The khronos
    ///   validation layer is added automatically when debug assertions are
    ///   enabled.
    /// * `features` - The physical device features required by the application.
    ///
    /// # Safety
    ///
    /// The application is responsible for synchronizing access to all Vulkan
    /// resources and destroying the render device at exit.
    pub unsafe fn create_render_device(
        &self,
        instance_extensions: &[String],
        instance_layers: &[String],
        features: PhysicalDeviceFeatures,
    ) -> Result<RenderDevice> {
        let instance =
            self.create_vulkan_instance(instance_extensions, instance_layers)?;
        RenderDevice::new(instance, features)
            .context("Unable to create the render device!")
    }

    /// Create a Vulkan instance with extensions and layers configured to
    /// such that it can present swapchain frames to the window.
    ///
    /// # Params
    ///
    /// * `instance_extensions` - Any extensions to enable when creating the
    ///   instance. Extensions for the swapchain on the current platform are
    ///   added automatically and do not need to be provided.
    /// * `instance_layers` - Any additional layers to provide. The khronos
    ///   validation layer is added automatically when debug assertions are
    ///   enabled.
    ///
    /// # Safety
    ///
    /// The application is responsible for synchronizing access to all Vulkan
    /// resources and destroying the Vulkan instance at exit.
    pub unsafe fn create_vulkan_instance(
        &self,
        instance_extensions: &[String],
        instance_layers: &[String],
    ) -> Result<VulkanInstance> {
        let mut all_instance_extensions =
            self.glfw.get_required_instance_extensions().context(
                "Cannot get the required instance extensions for this platform",
            )?;
        all_instance_extensions.extend_from_slice(instance_extensions);

        let mut all_layers = instance_layers.to_vec();
        if cfg!(debug_assertions) {
            all_layers.push("VK_LAYER_KHRONOS_validation".to_owned());
        }

        unsafe {
            VulkanInstance::new(&all_instance_extensions, &all_layers)
                .context("Error createing the Vulkan instance!")
        }
    }
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

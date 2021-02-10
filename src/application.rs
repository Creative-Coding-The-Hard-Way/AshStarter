//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

mod device;
mod instance;
mod window_surface;

use crate::application::{
    device::Device, instance::Instance, window_surface::WindowSurface,
};

use anyhow::{bail, Context, Result};
use ash::vk;
use glfw::Glfw;
use std::sync::{mpsc::Receiver, Arc};

/// The application's state.
pub struct Application {
    glfw: Glfw,
    window: glfw::Window,
    events: Option<Receiver<(f64, glfw::WindowEvent)>>,

    #[allow(dead_code)]
    instance: Arc<Instance>,

    #[allow(dead_code)]
    device: Arc<Device>,

    #[allow(dead_code)]
    window_surface: Arc<WindowSurface>,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("unable to initialize the glfw library")?;
        let (window, events) = Self::create_window(&mut glfw)?;
        let instance =
            Instance::new(&glfw.get_required_instance_extensions().context(
                "unable to get required vulkan extensions for this platform",
            )?)?;
        let window_surface = WindowSurface::new(&window, instance.clone())?;
        let device = Device::new(&instance)?;

        device.name_vulkan_object(
            "main application surface",
            vk::ObjectType::SURFACE_KHR,
            &window_surface.surface,
        )?;

        Ok(Self {
            glfw,
            window,
            events: Some(events),
            instance,
            device,
            window_surface,
        })
    }

    /// Create this application's glfw window
    fn create_window(
        glfw: &mut Glfw,
    ) -> Result<(glfw::Window, Receiver<(f64, glfw::WindowEvent)>)> {
        if !glfw.vulkan_supported() {
            bail!("vulkan is not supported on this device!");
        }
        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.create_window(1366, 768, "Ash Starter", glfw::WindowMode::Windowed)
            .context("unable to create the glfw window")
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        self.main_loop()?;
        Ok(())
    }

    /// Main window event loop. Events are dispatched via handle_event.
    fn main_loop(&mut self) -> Result<()> {
        self.window.set_key_polling(true);

        let events =
            self.events.take().context("event reciever is missing?!")?;

        while !self.window.should_close() {
            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                log::debug!("{:?}", event);
                self.handle_event(event)?;
            }
        }
        Ok(())
    }

    /// Handle window events and update the application state as needed.
    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        match event {
            glfw::WindowEvent::Key(
                glfw::Key::Escape,
                _,
                glfw::Action::Press,
                _,
            ) => {
                self.window.set_should_close(true);
            }

            _ => {}
        }

        Ok(())
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        log::debug!("cleanup application");
    }
}

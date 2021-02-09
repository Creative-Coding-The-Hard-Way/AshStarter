//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

mod instance;

use anyhow::{bail, Context, Result};
use ash::{version::InstanceV1_0, Entry, Instance};
use glfw::Glfw;
use std::sync::mpsc::Receiver;

/// The application's state.
pub struct Application {
    glfw: Glfw,
    window: glfw::Window,
    events: Option<Receiver<(f64, glfw::WindowEvent)>>,
    entry: Entry,
    instance: Instance,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("unable to initialize the glfw library")?;

        if !glfw.vulkan_supported() {
            bail!("vulkan is not supported on this device!");
        }
        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        let (mut window, events) = glfw
            .create_window(1366, 768, "Ash Starter", glfw::WindowMode::Windowed)
            .context("unable to create the glfw window")?;

        window.set_key_polling(true);

        let (instance, entry) = instance::create_instance(
            &glfw
                .get_required_instance_extensions()
                .context("unable to get the required vulkan extensions")?,
        )?;

        Ok(Self {
            glfw,
            window,
            events: Some(events),
            entry,
            instance,
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        self.main_loop()?;
        Ok(())
    }

    /// Main window event loop. Events are dispatched via handle_event.
    fn main_loop(&mut self) -> Result<()> {
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
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

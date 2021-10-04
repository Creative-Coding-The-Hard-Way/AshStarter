//! This module defines the main application initialization, event loop, and
//! rendering.
use anyhow::{bail, Context, Result};

use std::sync::mpsc::Receiver;

pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

// The main application state.
pub struct Application {
    /// The glfw library instance
    glfw: glfw::Glfw,

    /// The glfw window
    window: glfw::Window,

    /// The event reciever. Usually consumed by the application's main loop.
    event_receiver: Option<EventReceiver>,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        // Initialize the GLFW library. NOTE: glfw is not SEND, so it must
        // always be used on the main thread.
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("unable to setup glfw for this application")?;

        // These are vulkan demos! If Vulkan isn't supported on the device,
        // then the only reasonable thing to do is crash.
        if !glfw.vulkan_supported() {
            bail!("vulkan is not supported on this device!");
        }

        // Tell GLFW not to bother setting up the OpenGL API because we're
        // using Vulkan!
        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));

        // Attempt to create a fullscreen application using the primary
        // monitor's physical size.
        // If this fails for any reason, fall back to creating a windowed
        // application.
        let window_title = "Example 1 - GLFW";
        let (mut window, event_receiver) = glfw
            .create_window(1366, 768, window_title, glfw::WindowMode::Windowed)
            .context("unable to create the glfw window")?;

        window.set_key_polling(true);

        Ok(Self {
            glfw,
            window,
            event_receiver: Some(event_receiver),
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.event_receiver.take().unwrap();
        // block the application on the window staying open
        while !self.window.should_close() {
            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&event_receiver) {
                self.handle_event(event)?;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match event {
            WindowEvent::Close => {
                self.window.set_should_close(true);
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.window.set_should_close(true);
            }
            _ => {}
        }
        Ok(())
    }
}

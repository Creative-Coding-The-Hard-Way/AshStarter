use anyhow::{bail, Context, Result};
use glfw::Glfw;
use std::sync::mpsc::Receiver;

/// This struct represents the application's main state.
pub struct Application {
    glfw: Glfw,
    window: glfw::Window,
    events: Option<Receiver<(f64, glfw::WindowEvent)>>,
}

impl Application {
    /// Build a new instance of the application. This is allowed to fail if
    /// anything goes wrong or cannot be created.
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

        Ok(Self {
            glfw,
            window,
            events: Some(events),
        })
    }

    /// Main application loop
    pub fn run(mut self) -> Result<()> {
        self.init_vulkan();
        self.main_loop()?;
        Ok(())
    }

    fn init_vulkan(&self) {}

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

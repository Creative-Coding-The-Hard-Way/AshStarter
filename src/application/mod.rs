//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

mod frame;
mod graphics_pipeline;

use crate::rendering::{glfw_window::GlfwWindow, Device, Swapchain};

pub use self::{frame::Frame, graphics_pipeline::GraphicsPipeline};

use anyhow::{Context, Result};
use std::sync::Arc;

pub struct Application {
    window_surface: Arc<GlfwWindow>,
    frame: Frame,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let window_surface = GlfwWindow::new(|glfw| {
            let (mut window, event_receiver) = glfw
                .create_window(
                    1366,
                    768,
                    "Ash Starter",
                    glfw::WindowMode::Windowed,
                )
                .context("unable to create the glfw window")?;

            window.set_resizable(true);
            window.set_key_polling(true);
            window.set_size_polling(true);

            Ok((window, event_receiver))
        })?;

        let device = Device::new(window_surface.clone())?;
        let swapchain =
            Swapchain::new(device.clone(), window_surface.clone(), None)?;

        let pipeline = GraphicsPipeline::new(&device, &swapchain)?;

        let frame = Frame::new(
            &device,
            &swapchain,
            &pipeline,
            &window_surface.instance,
        )?;

        Ok(Self {
            window_surface,
            frame,
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        self.main_loop()?;
        Ok(())
    }

    /// Main window event loop. Events are dispatched via handle_event.
    fn main_loop(&mut self) -> Result<()> {
        let events = self
            .window_surface
            .event_receiver
            .borrow_mut()
            .take()
            .unwrap();

        while !self.window_surface.window.borrow().should_close() {
            self.window_surface.glfw.borrow_mut().poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                log::debug!("{:?}", event);
                self.handle_event(event)?;
            }
            self.frame.draw_frame()?;
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
                self.window_surface
                    .window
                    .borrow_mut()
                    .set_should_close(true);
            }

            glfw::WindowEvent::FramebufferSize(_, _) => {
                log::info!("resized");
                self.frame.needs_rebuild();
            }

            _ => {}
        }

        Ok(())
    }
}

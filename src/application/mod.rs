use {anyhow::Result, glfw::WindowEvent};

mod error;
mod glfw_window;

pub use {self::glfw_window::GlfwWindow, error::ApplicationError};

/// Applications can have state.
pub trait State {
    /// Create a new instance of this state.
    fn new(glfw_window: &mut GlfwWindow) -> Result<Self>
    where
        Self: Sized;

    /// Handle a GLFW event and update the application state.
    fn handle_event(
        &mut self,
        _glfw_window: &mut GlfwWindow,
        _window_event: WindowEvent,
    ) -> Result<()> {
        Ok(())
    }

    /// Called each time through the main application loop after all events
    /// have been processed.
    fn update(&mut self, _glfw_window: &mut GlfwWindow) -> Result<()> {
        Ok(())
    }
}

/// The main application state.
pub struct Application<S: State> {
    state: S,

    paused: bool,
    window: GlfwWindow,
}

impl<S> Application<S>
where
    S: Sized + State,
{
    /// Create a new running application.
    pub fn new(window_title: impl AsRef<str>) -> Result<Self> {
        let mut window = GlfwWindow::new(window_title)?;

        // Framebuffer polling is required for detecting when the app should be
        // paused.
        window.window_handle.set_framebuffer_size_polling(true);

        Ok(Self {
            state: S::new(&mut window)?,
            paused: false,
            window,
        })
    }

    /// Run the application until exit.
    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.window.event_receiver.take().unwrap();
        while !self.window.window_handle.should_close() {
            self.window.glfw.poll_events();
            for (_, window_event) in glfw::flush_messages(&event_receiver) {
                self.handle_event(window_event)?;
            }
            if !self.paused {
                self.state.update(&mut self.window)?;
            }
        }
        Ok(())
    }

    /// Handle a GLFW window event.
    fn handle_event(&mut self, window_event: WindowEvent) -> Result<()> {
        match window_event {
            WindowEvent::Close => {
                self.window.window_handle.set_should_close(true);
            }
            WindowEvent::FramebufferSize(width, height) => {
                self.paused = width == 0 || height == 0;
            }
            _ => (),
        }

        self.state.handle_event(&mut self.window, window_event)
    }
}

//! Provides structures for running a stateful single-window GLFW application.

use {anyhow::Result, glfw::WindowEvent};

mod glfw_window;
mod logging;

pub use self::glfw_window::GlfwWindow;

/// Application state can be any type which implements the State trait.
///
/// State is created after the GLFW window is created, but is allowed to
/// configure the window for things like resizability and event polling.
pub trait State {
    /// Create a new instance of this state.
    ///
    /// # Params
    ///
    /// * `window` - A fully constructed application window. The implementation
    ///   can use this handle to resize the window, apply GLFW window hints,
    ///   toggle fullscren, and construct a Vulkan instance which can present
    ///   surfaces to the window.
    fn new(window: &mut GlfwWindow) -> Result<Self>
    where
        Self: Sized;

    /// Handle a GLFW event and update the application state.
    ///
    /// # Params
    ///
    /// * `window` - The fully constructed application window. The application
    ///   can exit by calling `set_should_close` on the window.
    /// * `window_event` - The event currently being processed by the window.
    fn handle_event(
        &mut self,
        _window: &mut GlfwWindow,
        _window_event: glfw::WindowEvent,
    ) -> Result<()> {
        Ok(())
    }

    /// Called each time through the main application loop after all events
    /// have been processed.
    ///
    /// Update is not called while an application is paused while minimized.
    ///
    /// # Params
    ///
    /// * `window` - The fully constructed application window. The application
    ///   can exit by calling `set_should_close` on the window.
    fn update(&mut self, _window: &mut GlfwWindow) -> Result<()> {
        Ok(())
    }
}

/// Every application is comprised of a State type and a GLFW window.
/// Applications automatically pause if they are minimized or the window is
/// resized such that there is no drawing area.
pub struct Application<S: State> {
    state: S,
    paused: bool,
    window: GlfwWindow,
}

// Public API

impl<S> Application<S>
where
    S: Sized + State,
{
    /// Create and run the Application until the window is closed.
    ///
    /// The window title is just the Application state struct's type name.
    pub fn run() -> Result<()> {
        let window_title = std::any::type_name::<S>();
        Self::new(window_title)?.main_loop()
    }
}

// Private API

impl<S> Application<S>
where
    S: Sized + State,
{
    /// Create a new running application.
    fn new(window_title: impl AsRef<str>) -> Result<Self> {
        self::logging::setup();

        let mut window = GlfwWindow::new(window_title)?;

        // Framebuffer polling is required for detecting when the app should be
        // paused.
        window.set_framebuffer_size_polling(true);

        Ok(Self {
            state: S::new(&mut window)?,
            paused: false,
            window,
        })
    }

    /// Run the application until until the window is closed.
    fn main_loop(mut self) -> Result<()> {
        let event_receiver = self.window.event_receiver.take().unwrap();
        while !self.window.should_close() {
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
                self.window.set_should_close(true);
            }
            WindowEvent::FramebufferSize(width, height) => {
                self.paused = width == 0 || height == 0;
            }
            _ => (),
        }

        self.state.handle_event(&mut self.window, window_event)
    }
}

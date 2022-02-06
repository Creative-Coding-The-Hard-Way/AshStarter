pub use self::{glfw_window::GlfwWindow, window_error::WindowError};

mod glfw_window;
mod window_error;

use std::sync::mpsc::Receiver;

/// GLFW uses a Receiver for accepting window events. This type alias is more
/// convenient to write/read than the full name.
pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

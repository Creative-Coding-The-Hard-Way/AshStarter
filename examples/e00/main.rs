use {
    anyhow::Result,
    ccthw::application::{Application, GlfwWindow, State},
};

struct AppLifecycle;

impl State for AppLifecycle {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);
        Ok(Self)
    }

    fn handle_event(
        &mut self,
        window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                window.set_should_close(true);
            }
            _ => (),
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    Application::<AppLifecycle>::run()
}

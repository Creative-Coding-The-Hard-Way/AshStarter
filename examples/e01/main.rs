use {
    anyhow::Result,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::RenderDevice,
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
    std::sync::Arc,
};

struct RenderDeviceExample {
    _render_device: Arc<RenderDevice>,
}

impl State for RenderDeviceExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);
        let render_device = unsafe {
            window.create_default_render_device(
                PhysicalDeviceFeatures::default(),
            )?
        };

        log::info!("Created render device: {}", render_device);

        Ok(Self {
            _render_device: render_device,
        })
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
    Application::<RenderDeviceExample>::run()
}

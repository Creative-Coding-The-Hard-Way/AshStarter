use {
    anyhow::Result,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::RenderDevice,
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
};

struct CreateRenderDevice {
    render_device: RenderDevice,
}

impl State for CreateRenderDevice {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);
        let render_device = unsafe {
            // SAFE because state owns the instance and calls destroy in it's
            // Drop impl.
            window.create_render_device(
                &[],
                &[],
                PhysicalDeviceFeatures::default(),
            )?
        };
        log::info!("Created Render Device: {}", render_device);
        Ok(Self { render_device })
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

impl Drop for CreateRenderDevice {
    fn drop(&mut self) {
        unsafe {
            // SAFE because there are no resources which depend on the instance.
            self.render_device.device().device_wait_idle().unwrap();
            self.render_device.destroy();
        }
    }
}

fn main() -> Result<()> {
    Application::<CreateRenderDevice>::run()
}

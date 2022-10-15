use {
    anyhow::Result,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{RenderDevice, Swapchain},
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
};

struct CreateRenderDevice {
    swapchain: Swapchain,
    render_device: RenderDevice,
}

impl State for CreateRenderDevice {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        log::info!("IS DEBUG ASSERT ENABLED {}", cfg!(debug_assertions));

        window.set_key_polling(true);
        let render_device = unsafe {
            // SAFE because the render device is destroyed when state is
            // dropped.
            window.create_default_render_device(
                PhysicalDeviceFeatures::default(),
            )?
        };
        let (w, h) = window.get_framebuffer_size();
        let swapchain = unsafe {
            Swapchain::new(&render_device, (w as u32, h as u32), None)?
        };
        log::info!("{}", swapchain);
        Ok(Self {
            swapchain,
            render_device,
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

impl Drop for CreateRenderDevice {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.destroy();
            self.render_device.destroy();
        }
    }
}

fn main() -> Result<()> {
    Application::<CreateRenderDevice>::run()
}

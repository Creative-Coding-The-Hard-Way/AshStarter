mod particles;

use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::{
        msaa_display::MSAADisplay,
        vulkan_api::{
            HostCoherentBuffer, PhysicalDeviceFeatures, RenderDevice,
        },
        AcquiredFrame,
    },
    logging,
};
use particles::{Graphics, Particle, SimulationConfig};

/// This example renders a gpu driven particle system using async
/// compute shaders to simulate particles.
struct Example11GPUParticles {
    simulation_config: SimulationConfig,
    graphics: Graphics,

    msaa_display: MSAADisplay,
    render_device: Arc<RenderDevice>,
}

impl State for Example11GPUParticles {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);
        let render_device =
            Arc::new(window.create_render_device_with_features(
                PhysicalDeviceFeatures {
                    maintenance4: vk::PhysicalDeviceMaintenance4Features {
                        maintenance4: vk::TRUE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |features| features.maintenance4.maintenance4 == vk::TRUE,
            )?);
        let msaa_display = MSAADisplay::new(render_device.clone(), window)?;
        let particles = Arc::new(HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            &[Particle {
                pos: [0.0, 0.0],
                vel: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            }],
        )?);
        let (w, h) = window.window_handle.get_framebuffer_size();
        let simulation_config =
            SimulationConfig::new(100.0, w as f32 / h as f32);
        let graphics = Graphics::new(
            &render_device,
            &msaa_display,
            particles,
            simulation_config,
        )?;
        Ok(Self {
            simulation_config,
            graphics,
            msaa_display,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        glfw_window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                glfw_window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                glfw_window.window_handle.set_should_close(true);
            }
            WindowEvent::FramebufferSize(_, _) => {
                self.msaa_display.invalidate_swapchain();
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        let mut frame = match self.msaa_display.begin_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return self.build_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            AcquiredFrame::Available(frame) => frame,
        };

        unsafe {
            self.msaa_display
                .begin_render_pass(&mut frame, [0.0, 0.0, 0.0, 1.0]);
            self.graphics.draw(
                frame.command_buffer(),
                self.msaa_display.swapchain_extent(),
            )?;
            frame.command_buffer().end_render_pass();
        }

        self.msaa_display.end_frame(frame)?;

        Ok(())
    }
}

impl Example11GPUParticles {
    fn build_swapchain_resources(
        &mut self,
        (width, height): (i32, i32),
    ) -> Result<()> {
        self.msaa_display
            .rebuild_swapchain_resources((width, height))?;

        self.simulation_config.resize(width as f32 / height as f32);

        // Safe because rebuilding MSAA display resources forces every frame
        // to finish rendering, so there is no possibility of graphics resources
        // being used by pending command buffers.
        unsafe {
            self.graphics
                .rebuild_swapchain_resources(&self.msaa_display)?;
            self.graphics
                .update_simulation_config(&self.simulation_config)?;
        };

        Ok(())
    }
}

impl Drop for Example11GPUParticles {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    logging::setup()?;
    Application::<Example11GPUParticles>::new("Example 10 - Compute")?.run()
}

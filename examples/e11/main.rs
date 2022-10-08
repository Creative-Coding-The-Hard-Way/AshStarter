mod particles;

use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::{
        msaa_display::MSAADisplay,
        vulkan_api::{DeviceLocalBuffer, PhysicalDeviceFeatures, RenderDevice},
        AcquiredFrame,
    },
    logging,
};
use particles::{
    Graphics, Initializer, Integrator, Particle, SimulationConfig,
};

/// This example renders a gpu driven particle system using async
/// compute shaders to simulate particles.
struct Example11GPUParticles {
    particles: Arc<DeviceLocalBuffer<Particle>>,
    simulation_config: SimulationConfig,
    graphics: Graphics,
    initializer: Initializer,
    integrator: Integrator,

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
        let msaa_display = MSAADisplay::new(
            render_device.clone(),
            window,
            vk::SampleCountFlags::TYPE_2,
        )?;
        let particles = Arc::new(DeviceLocalBuffer::<Particle>::new(
            render_device.clone(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            1_000_000,
        )?);
        let (w, h) = window.window_handle.get_framebuffer_size();
        let simulation_config =
            SimulationConfig::new(100.0, w as f32 / h as f32);
        let graphics = Graphics::new(
            &render_device,
            &msaa_display,
            particles.clone(),
            simulation_config,
        )?;

        let mut initializer = Initializer::new(&render_device)?;

        // safe because initialization is synchronous and nothing is using the
        // particle buffer yet
        unsafe {
            initializer.initialize_particles(&particles, simulation_config)?
        };

        let integrator = Integrator::new(&render_device)?;

        Ok(Self {
            simulation_config,
            graphics,
            initializer,
            integrator,
            particles,

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
            WindowEvent::Key(Key::Enter, _, Action::Release, _) => unsafe {
                self.render_device.wait_idle()?;
                self.initializer.initialize_particles(
                    &self.particles,
                    self.simulation_config,
                )?;
            },
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
            self.render_device.wait_idle()?;

            // Safe(ish) because this function stalls the gpu until the
            // integration finishes.
            self.integrator
                .integrate_particles(&self.particles, self.simulation_config)?
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

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
use particles::{Graphics, Integrator, SimulationConfig};

/// This example renders a gpu driven particle system using async
/// compute shaders to simulate particles.
struct Example11GPUParticles {
    needs_initialized: bool,
    mouse_pos: (f32, f32),
    left_mouse_button_pressed: bool,
    right_mouse_button_pressed: bool,

    simulation_config: SimulationConfig,
    graphics: Graphics,
    integrator: Integrator,

    msaa_display: MSAADisplay,
    render_device: Arc<RenderDevice>,
}

impl State for Example11GPUParticles {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);
        window.window_handle.set_cursor_pos_polling(true);
        window.window_handle.set_mouse_button_polling(true);
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
        let (w, h) = window.window_handle.get_framebuffer_size();

        let simulation_config =
            SimulationConfig::new(100.0, w as f32 / h as f32, 4_000_000);
        let particles = Arc::new(DeviceLocalBuffer::new(
            render_device.clone(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            simulation_config.particle_count() as usize,
        )?);

        let graphics = Graphics::new(
            render_device.clone(),
            &msaa_display,
            simulation_config,
            particles.clone(),
        )?;

        let integrator = Integrator::new(
            &render_device,
            &[
                include_bytes!("./shaders/initialize.comp.spv"),
                include_bytes!("./shaders/integrate.comp.spv"),
            ],
            simulation_config,
            particles,
        )?;

        Ok(Self {
            needs_initialized: true,
            mouse_pos: (0.0, 0.0),
            left_mouse_button_pressed: false,
            right_mouse_button_pressed: false,
            simulation_config,
            graphics,
            integrator,
            msaa_display,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        glfw_window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{
            Action, Key, MouseButtonLeft, MouseButtonRight, WindowEvent,
        };
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                glfw_window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                glfw_window.window_handle.set_should_close(true);
            }
            WindowEvent::Key(Key::Enter, _, Action::Release, _) => {
                self.needs_initialized = true;
            }
            WindowEvent::FramebufferSize(_, _) => {
                self.msaa_display.invalidate_swapchain();
            }
            WindowEvent::CursorPos(x, y) => {
                let display_extent = self.msaa_display.swapchain_extent();
                let unit_x = x as f32 / display_extent.width as f32;
                let unit_y = y as f32 / display_extent.height as f32;
                let norm_x = (unit_x * 2.0) - 1.0;
                let norm_y = (unit_y * -2.0) + 1.0;
                let x = norm_x * self.simulation_config.width() / 2.0;
                let y = norm_y * self.simulation_config.height() / 2.0;
                self.mouse_pos = (x, y);
            }
            WindowEvent::MouseButton(MouseButtonLeft, Action::Press, _) => {
                self.left_mouse_button_pressed = true;
            }
            WindowEvent::MouseButton(MouseButtonLeft, Action::Release, _) => {
                self.left_mouse_button_pressed = false;
            }
            WindowEvent::MouseButton(MouseButtonRight, Action::Press, _) => {
                self.right_mouse_button_pressed = true;
            }
            WindowEvent::MouseButton(MouseButtonRight, Action::Release, _) => {
                self.right_mouse_button_pressed = false;
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

        let compute_shader_index = if self.needs_initialized {
            self.needs_initialized = false;
            0 // the initialize shader is provided first in new()
        } else {
            1 // the integrate shader is provided second in new()
        };
        unsafe {
            self.integrator.integrate_particles(
                frame.command_buffer(),
                compute_shader_index,
                self.mouse_pos,
                self.left_mouse_button_pressed,
                self.right_mouse_button_pressed,
            )?;

            self.msaa_display
                .begin_render_pass(&mut frame, [0.0, 0.0, 0.0, 1.0]);
            self.graphics
                .draw(&mut frame, self.msaa_display.swapchain_extent())?;
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

        unsafe {
            // Safe because msaa_display waits for all frame command buffers to
            // finish executing before rebuilding swapchain resources.
            self.graphics
                .rebuild_swapchain_resources(&self.msaa_display)?;
        }

        self.simulation_config.resize(width as f32 / height as f32);
        unsafe {
            // Safe because msaa_display waits for all frame command buffers to
            // finish executing before rebuilding swapchain resources.
            self.integrator
                .update_simulation_config(&self.simulation_config)?;
            self.graphics
                .update_simulation_config(&self.simulation_config)?;
        }

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
    let _logger = logging::setup()?;
    Application::<Example11GPUParticles>::new("Example 11 - GPU Particles")?
        .run()
}

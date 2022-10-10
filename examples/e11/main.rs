mod particles;

use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use ash::vk;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::{
        msaa_display::MSAADisplay,
        vulkan_api::{
            DeviceLocalBuffer, PhysicalDeviceFeatures, RenderDevice,
            VulkanError,
        },
        AcquiredFrame,
    },
    logging,
};
use particles::{Graphics, Integrator, Particle, SimulationConfig};

struct SynchronizedBuffer {
    buffer: DeviceLocalBuffer<Particle>,
    frames_in_flight: HashSet<usize>,
    is_write_target: bool,
}

impl SynchronizedBuffer {
    fn new(
        render_device: Arc<RenderDevice>,
        particle_count: usize,
    ) -> Result<Self, VulkanError> {
        Ok(Self {
            frames_in_flight: HashSet::with_capacity(3),
            buffer: DeviceLocalBuffer::new(
                render_device,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                particle_count,
            )?,
            is_write_target: false,
        })
    }

    fn reserve_for_frame(&mut self, frame_index: usize) {
        debug_assert!(!self.is_write_target);
        self.frames_in_flight.insert(frame_index);
    }

    fn free_for_frame(&mut self, frame_index: usize) {
        self.frames_in_flight.remove(&frame_index);
    }

    fn is_free(&self) -> bool {
        self.frames_in_flight.is_empty()
    }

    fn reserve_write_target(&mut self) {
        self.is_write_target = true;
    }

    fn release_write_target(&mut self) {
        self.is_write_target = false;
    }

    fn is_done_writing(&self) -> bool {
        !self.is_write_target
    }
}

struct TrippleBufferedParticles {
    buffers: [SynchronizedBuffer; 3],
    read_buffer_index: usize,
    write_buffer_index: usize,
    scratch_buffer_index: usize,
}

impl TrippleBufferedParticles {
    fn new(
        render_device: &Arc<RenderDevice>,
        particle_count: usize,
    ) -> Result<Self, VulkanError> {
        Ok(Self {
            buffers: [
                SynchronizedBuffer::new(render_device.clone(), particle_count)?,
                SynchronizedBuffer::new(render_device.clone(), particle_count)?,
                SynchronizedBuffer::new(render_device.clone(), particle_count)?,
            ],
            read_buffer_index: 0,
            write_buffer_index: 1,
            scratch_buffer_index: 2,
        })
    }

    fn swap_buffers(&mut self) {
        debug_assert!(self.buffers[self.write_buffer_index].is_done_writing());
        let new_read_buffer_index = self.write_buffer_index;
        let new_write_buffer_index = self.scratch_buffer_index;
        let new_scratch_buffer_index = self.read_buffer_index;
        self.read_buffer_index = new_read_buffer_index;
        self.write_buffer_index = new_write_buffer_index;
        self.scratch_buffer_index = new_scratch_buffer_index;
    }

    fn read_buffer(&self) -> &SynchronizedBuffer {
        &self.buffers[self.read_buffer_index]
    }

    fn read_buffer_mut(&mut self) -> &mut SynchronizedBuffer {
        &mut self.buffers[self.read_buffer_index]
    }

    fn write_buffer(&self) -> &SynchronizedBuffer {
        &self.buffers[self.write_buffer_index]
    }

    fn write_buffer_mut(&mut self) -> &mut SynchronizedBuffer {
        &mut self.buffers[self.write_buffer_index]
    }

    fn scratch_buffer_mut(&mut self) -> &mut SynchronizedBuffer {
        &mut self.buffers[self.scratch_buffer_index]
    }
}

/// This example renders a gpu driven particle system using async
/// compute shaders to simulate particles.
struct Example11GPUParticles {
    needs_initialized: bool,

    simulation_config: SimulationConfig,
    particles: TrippleBufferedParticles,
    graphics: Graphics,
    integrator: Integrator,

    mouse_pos: (f32, f32),
    pressed: bool,

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
            SimulationConfig::new(100.0, w as f32 / h as f32, 8_000_000);
        let particles = TrippleBufferedParticles::new(
            &render_device,
            simulation_config.particle_count() as usize,
        )?;

        let graphics = Graphics::new(
            render_device.clone(),
            &msaa_display,
            simulation_config,
        )?;

        let integrator = Integrator::new(
            &render_device,
            &[
                include_bytes!("./shaders/initialize.comp.spv"),
                include_bytes!("./shaders/integrate.comp.spv"),
            ],
            simulation_config,
        )?;

        Ok(Self {
            needs_initialized: true,

            simulation_config,
            particles,
            graphics,
            integrator,
            mouse_pos: (0.0, 0.0),
            pressed: false,

            msaa_display,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        glfw_window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, MouseButtonLeft, WindowEvent};
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
                self.pressed = true;
            }
            WindowEvent::MouseButton(MouseButtonLeft, Action::Release, _) => {
                self.pressed = false;
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        if self.integrator.is_integration_finished()?
            && self.particles.write_buffer().is_free()
        {
            if self.particles.write_buffer().is_done_writing() {
                // the write hasn't started
                self.particles.write_buffer_mut().reserve_write_target();
                unsafe {
                    self.integrator
                        .set_read_buffer(&self.particles.read_buffer().buffer);
                    self.integrator.set_write_buffer(
                        &self.particles.write_buffer().buffer,
                    );
                    if self.needs_initialized {
                        self.integrator.integrate_particles(
                            0,
                            self.mouse_pos,
                            self.pressed,
                        )?;
                        self.needs_initialized = false;
                    } else {
                        self.integrator.integrate_particles(
                            1,
                            self.mouse_pos,
                            self.pressed,
                        )?;
                    }
                }
            } else {
                // Integration has finished, but the write buffer is still
                // marked, this means an iteration has completed.
                self.particles.write_buffer_mut().release_write_target();
                self.particles.swap_buffers();
            }
        }

        let mut frame = match self.msaa_display.begin_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return self.build_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            AcquiredFrame::Available(frame) => frame,
        };

        let frame_index = frame.swapchain_image_index();
        self.particles
            .scratch_buffer_mut()
            .free_for_frame(frame_index);
        self.particles
            .write_buffer_mut()
            .free_for_frame(frame_index);
        self.particles
            .read_buffer_mut()
            .reserve_for_frame(frame_index);

        unsafe {
            self.msaa_display
                .begin_render_pass(&mut frame, [0.0, 0.0, 0.0, 1.0]);

            self.graphics.set_read_buffer(
                frame_index,
                &self.particles.read_buffer().buffer,
            );
            self.graphics.draw(
                &mut frame,
                self.msaa_display.swapchain_extent(),
                self.integrator.time_since_last_update() * 0.0,
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
        self.integrator.wait_for_integration_to_complete()?;

        self.simulation_config.resize(width as f32 / height as f32);

        // Safe because rebuilding MSAA display resources forces every frame
        // to finish rendering, so there is no possibility of graphics resources
        // being used by pending command buffers.
        unsafe {
            self.graphics
                .rebuild_swapchain_resources(&self.msaa_display)?;
            self.graphics
                .update_simulation_config(&self.simulation_config)?;
            self.integrator
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

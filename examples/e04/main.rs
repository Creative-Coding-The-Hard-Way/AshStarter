use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            ColorPass, FrameStatus, FramesInFlight, RenderDevice,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
};

struct RenderPassExample {
    color_pass: ColorPass,
    frames_in_flight: FramesInFlight,
    render_device: RenderDevice,
}

impl State for RenderPassExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);

        let render_device = unsafe {
            // SAFE because the render device is destroyed when state is
            // dropped.
            let mut device_features = PhysicalDeviceFeatures::default();
            // enable synchronization2 for queue_submit2
            device_features.vulkan_13_features_mut().synchronization2 =
                vk::TRUE;
            window.create_default_render_device(device_features)?
        };

        let frames_in_flight = unsafe {
            // SAFE because the render device is destroyed when state is dropped
            FramesInFlight::new(
                &render_device,
                window.get_framebuffer_size(),
                3,
            )?
        };

        let color_pass = unsafe {
            ColorPass::new(
                &render_device,
                frames_in_flight.swapchain().images(),
                frames_in_flight.swapchain().image_format(),
                frames_in_flight.swapchain().extent(),
            )?
        };

        Ok(Self {
            color_pass,
            frames_in_flight,
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

    fn update(&mut self, window: &mut GlfwWindow) -> Result<()> {
        let frame =
            match self.frames_in_flight.acquire_frame(&self.render_device)? {
                FrameStatus::FrameAcquired(frame) => frame,
                FrameStatus::SwapchainNeedsRebuild => {
                    return self.rebuild_swapchain(window);
                }
            };

        unsafe {
            self.color_pass.begin_render_pass(
                &self.render_device,
                frame.command_buffer(),
                vk::SubpassContents::INLINE,
                frame.swapchain_image_index(),
                [0.5, 0.0, 0.0, 1.0],
            );

            // draw commands go here

            self.render_device
                .cmd_end_render_pass(frame.command_buffer());
        }

        self.frames_in_flight
            .present_frame(&self.render_device, frame)?;

        Ok(())
    }
}

impl RenderPassExample {
    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        unsafe {
            self.frames_in_flight.stall_and_rebuild_swapchain(
                &self.render_device,
                window.get_framebuffer_size(),
            )?;

            self.color_pass.destroy(&self.render_device);
            self.color_pass = ColorPass::new(
                &self.render_device,
                self.frames_in_flight.swapchain().images(),
                self.frames_in_flight.swapchain().image_format(),
                self.frames_in_flight.swapchain().extent(),
            )?;
        };

        Ok(())
    }
}

impl Drop for RenderPassExample {
    fn drop(&mut self) {
        unsafe {
            self.frames_in_flight
                .wait_for_all_frames_to_complete(&self.render_device)
                .expect("Error waiting for all frame operations to complete");
            self.color_pass.destroy(&self.render_device);
            self.frames_in_flight.destroy(&self.render_device);
            self.render_device.destroy();
        }
    }
}

fn main() -> Result<()> {
    Application::<RenderPassExample>::run()
}

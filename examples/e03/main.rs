use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            FrameStatus, FramesInFlight, RenderDevice, Swapchain,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
    std::sync::Arc,
};

struct FramesInFlightExample {
    frames_in_flight: FramesInFlight,
    render_device: Arc<RenderDevice>,
}

impl State for FramesInFlightExample {
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
                render_device.clone(),
                window.get_framebuffer_size(),
                3,
            )?
        };

        Ok(Self {
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
        let frame = match self.frames_in_flight.acquire_frame()? {
            FrameStatus::FrameAcquired(frame) => frame,
            FrameStatus::SwapchainNeedsRebuild => {
                return self.rebuild_swapchain(window);
            }
        };

        // use a image memory barrier to transition the swapchain image layout
        // to present_src_khr
        unsafe {
            let image_memory_barriers = [vk::ImageMemoryBarrier2 {
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                image: self.swapchain().images()[frame.swapchain_image_index()],
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            }];
            let dependency_info = vk::DependencyInfo {
                memory_barrier_count: 0,
                p_memory_barriers: std::ptr::null(),
                buffer_memory_barrier_count: 0,
                p_buffer_memory_barriers: std::ptr::null(),
                p_image_memory_barriers: image_memory_barriers.as_ptr(),
                image_memory_barrier_count: image_memory_barriers.len() as u32,
                ..Default::default()
            };
            self.render_device.device().cmd_pipeline_barrier2(
                frame.command_buffer(),
                &dependency_info,
            );
        };

        self.frames_in_flight.present_frame(frame)?;

        Ok(())
    }
}

impl FramesInFlightExample {
    /// Get a reference to the current swapchain.
    fn swapchain(&self) -> &Swapchain {
        self.frames_in_flight.swapchain()
    }

    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        unsafe {
            self.frames_in_flight
                .stall_and_rebuild_swapchain(window.get_framebuffer_size())?
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    Application::<FramesInFlightExample>::run()
}

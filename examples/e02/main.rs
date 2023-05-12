use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            raii, RenderDevice, Swapchain, SwapchainStatus,
        },
    },
    ccthw_ash_instance::{PhysicalDeviceFeatures, VulkanHandle},
    std::sync::Arc,
};

struct CreateSwapchainExample {
    swapchain_needs_rebuild: bool,
    command_pool: raii::CommandPool,
    acquire_semaphore: raii::Semaphore,
    release_semaphore: raii::Semaphore,
    swapchain: Option<Swapchain>,
    render_device: Arc<RenderDevice>,
}

impl State for CreateSwapchainExample {
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

        let (w, h) = window.get_framebuffer_size();
        let swapchain = unsafe {
            Swapchain::new(render_device.clone(), (w as u32, h as u32), None)?
        };
        log::info!("{}", swapchain);

        let acquire_semaphore =
            unsafe { raii::Semaphore::new(render_device.clone())? };
        let release_semaphore =
            unsafe { raii::Semaphore::new(render_device.clone())? };

        let mut command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo {
                queue_family_index: render_device
                    .graphics_queue()
                    .family_index(),
                ..Default::default()
            };
            raii::CommandPool::new(render_device.clone(), &create_info)?
        };
        command_pool.allocate_primary_command_buffers(1)?;

        Ok(Self {
            swapchain_needs_rebuild: false,
            command_pool,
            acquire_semaphore,
            release_semaphore,
            swapchain: Some(swapchain),
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
        if self.swapchain_needs_rebuild {
            return self.rebuild_swapchain(window);
        }

        // Acquire Swapchain image
        // -----------------------

        let index = unsafe {
            let result = self.swapchain().acquire_swapchain_image(
                self.acquire_semaphore.raw(),
                vk::Fence::null(),
            )?;
            match result {
                SwapchainStatus::Index(index) => index,
                SwapchainStatus::NeedsRebuild => {
                    self.swapchain_needs_rebuild = true;
                    return Ok(());
                }
            }
        };

        // Build frame command buffer
        // --------------------------

        unsafe {
            self.render_device.device().reset_command_pool(
                self.command_pool.raw(),
                vk::CommandPoolResetFlags::empty(),
            )?;
        }

        // begin the command buffer
        let command_buffer = self.command_pool.primary_command_buffer(0);
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.render_device
                .device()
                .begin_command_buffer(command_buffer, &begin_info)?
        };

        // use a image memory barrier to transition the swapchain image layout
        // to present_src_khr
        unsafe {
            let image_memory_barriers = [vk::ImageMemoryBarrier2 {
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                image: self.swapchain().images()[index],
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
            self.render_device
                .device()
                .cmd_pipeline_barrier2(command_buffer, &dependency_info);
        };

        // end the command buffer and submit
        unsafe {
            self.render_device
                .device()
                .end_command_buffer(command_buffer)?;
            let command_buffer_infos = [vk::CommandBufferSubmitInfo {
                command_buffer,
                ..Default::default()
            }];
            let wait_infos = [vk::SemaphoreSubmitInfo {
                semaphore: self.acquire_semaphore.raw(),
                stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }];
            let signal_infos = [vk::SemaphoreSubmitInfo {
                semaphore: self.release_semaphore.raw(),
                stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }];
            let submit_info = vk::SubmitInfo2 {
                p_wait_semaphore_infos: wait_infos.as_ptr(),
                wait_semaphore_info_count: wait_infos.len() as u32,
                p_command_buffer_infos: command_buffer_infos.as_ptr(),
                command_buffer_info_count: command_buffer_infos.len() as u32,
                p_signal_semaphore_infos: signal_infos.as_ptr(),
                signal_semaphore_info_count: signal_infos.len() as u32,
                ..Default::default()
            };
            self.render_device.device().queue_submit2(
                *self.render_device.graphics_queue().raw(),
                &[submit_info],
                vk::Fence::null(),
            )?;
        }

        // Present the swapchain image
        // ---------------------------

        unsafe {
            let status = self.swapchain().present_swapchain_image(
                index,
                &[self.release_semaphore.raw()],
            )?;
            if status == SwapchainStatus::NeedsRebuild {
                self.swapchain_needs_rebuild = true;
            }
        }

        // Stall the GPU every frame. This is excessively slow, but makes
        // synchronization trivial.
        unsafe { self.render_device.device().device_wait_idle()? };

        Ok(())
    }
}

impl CreateSwapchainExample {
    /// Get a reference to the current swapchain.
    fn swapchain(&self) -> &Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        // Wait for all pending operations to complete before rebuilding the
        // swapchain.
        unsafe { self.render_device.device().device_wait_idle()? };

        let (w, h) = window.get_framebuffer_size();
        self.swapchain = unsafe {
            Some(Swapchain::new(
                self.render_device.clone(),
                (w as u32, h as u32),
                self.swapchain.take(),
            )?)
        };

        log::debug!("Built New Swapchain - {:#?}", self.swapchain());

        self.swapchain_needs_rebuild = false;

        Ok(())
    }
}

impl Drop for CreateSwapchainExample {
    fn drop(&mut self) {
        unsafe {
            let device = self.render_device.device();
            // Wait for all pending operations to complete before destroying
            // anything.
            device.device_wait_idle().expect(
                "Error waiting for pending graphics operations to complete!",
            );
        }
    }
}

fn main() -> Result<()> {
    Application::<CreateSwapchainExample>::run()
}

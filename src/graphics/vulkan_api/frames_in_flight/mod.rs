mod frame;
mod frame_sync;

use {
    self::frame_sync::FrameSync,
    super::{RenderDevice, SwapchainStatus},
    crate::graphics::{vulkan_api::Swapchain, GraphicsError},
    anyhow::Context,
    ash::vk,
    ccthw_ash_instance::VulkanHandle,
    std::sync::Arc,
};

pub use self::frame::Frame;

/// The result of a call to FramesInFlight::acquire_frame.
pub enum FrameStatus {
    /// The frame is acquired and ready for commands.
    FrameAcquired(Frame),

    /// No frame could be acquired because the swapchain needs to be rebuilt.
    SwapchainNeedsRebuild,
}

/// A utility for synchronizing graphics commands and submission for multiple
/// in-flight frames.
pub struct FramesInFlight {
    swapchain_needs_rebuild: bool,
    current_frame: usize,
    frames: Vec<Option<FrameSync>>,
    swapchain: Option<Swapchain>,
    render_device: Arc<RenderDevice>,
}

impl FramesInFlight {
    /// Create resources for synchronizing multiple in-flight frames.
    ///
    /// # Params
    ///
    /// * `render_device` - used to create all Vulkan resources
    /// * `framebuffer_size` - the size of the framebuffer in pixels. This is
    ///   used to create the swapchain and acompanying images.
    /// * `frame_count` - the number of in-flight frames to support. Typically
    ///   this is 2 for double-buffering or 3 for triple-buffering in-filght
    ///   frames.
    ///
    /// # Safety
    ///
    /// Unsafe because the application must destroy this struct prior to
    /// exiting. Furthermore, destruction of Vulkan resources which are used
    /// by in-flight frames should be delayed until all frames have finished
    /// executing or until the device is idle.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        framebuffer_size: (i32, i32),
        frame_count: usize,
    ) -> Result<Self, GraphicsError> {
        let mut frames = vec![];
        for i in 0..frame_count {
            frames.push(unsafe {
                // SAFE because all frames are kept and destroyed by this
                // struct.
                Some(FrameSync::new(render_device.clone(), i)?)
            });
        }

        let (w, h) = framebuffer_size;
        let swapchain = unsafe {
            // SAFE because the swapchain is kept and destroyed by this struct.
            Swapchain::new(render_device.clone(), (w as u32, h as u32), None)?
        };

        Ok(Self {
            swapchain_needs_rebuild: false,
            current_frame: 0,
            frames,
            swapchain: Some(swapchain),
            render_device,
        })
    }

    /// Wait for every frame's commands to finish executing on the GPU.
    ///
    /// # Params
    ///
    /// * `render_device` - the render device used to create the frames in
    ///   flight.
    ///
    /// # Safety
    ///
    /// It is an error to wait for frames while recording commands for a frame.
    /// e.g. do not call this function between calls to `acquire_frame` and
    /// `present_frame`.
    pub unsafe fn wait_for_all_frames_to_complete(
        &self,
    ) -> Result<(), GraphicsError> {
        for (index, frame_sync) in self.frames.iter().enumerate() {
            frame_sync
                .as_ref()
                .with_context(|| {
                    format!("Unable to acquire frame {} while waiting for frames to complete!", index)})?
                .wait_for_graphics_commands_to_complete()
                .with_context(|| {
                    format!(
                        "Error waiting for frame {index}'s commands to complete"
                    )
                })?;
        }
        Ok(())
    }

    /// Wait for every frame to finish executing then rebuild the swapchain.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - it is invalid to rebuild the swapchain while recording a frame. In
    ///    other words, never call this function after getting a frame from
    ///    `acquire_frame` and before returning that frame with `present_frame`.
    pub unsafe fn stall_and_rebuild_swapchain(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<(), GraphicsError> {
        self.wait_for_all_frames_to_complete()?;

        let old_swapchain = self.swapchain.take();
        let (w, h) = framebuffer_size;
        let new_swapchain = Swapchain::new(
            self.render_device.clone(),
            (w as u32, h as u32),
            old_swapchain,
        )?;
        self.swapchain = Some(new_swapchain);

        self.swapchain_needs_rebuild = false;

        Ok(())
    }

    /// Get the current swapchain.
    pub fn swapchain(&self) -> &Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    /// Manually invalidate the swapchain so it is forced to be rebuilt the next
    /// time a frame is requested.
    ///
    /// This can be useful in cases where it's known that the swapchain will
    /// need to be rebuilt (like when the application window is resized).
    pub fn invalidate_swapchain(&mut self) {
        self.swapchain_needs_rebuild = true;
    }

    /// The maximum number of in-flight frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Acquire the next frame for rendering.
    ///
    /// # Params
    ///
    /// * `render_device` - the render device used to create the frames in
    ///   flight.
    pub fn acquire_frame(&mut self) -> Result<FrameStatus, GraphicsError> {
        if self.swapchain_needs_rebuild {
            return Ok(FrameStatus::SwapchainNeedsRebuild);
        }

        // advance the frame counter
        self.current_frame = (self.current_frame + 1) % self.frames.len();

        // grab the synchronization resources for the current in-flight frame.
        let frame_sync =
            self.frames[self.current_frame].take().with_context(|| {
                format!("Unable to acquire frame {}", self.current_frame)
            })?;

        let result = unsafe {
            self.swapchain().acquire_swapchain_image(
                frame_sync.swapchain_image_acquired_semaphore.raw(),
                vk::Fence::null(),
            )?
        };
        let swapchain_image_index = match result {
            SwapchainStatus::Index(index) => index,
            SwapchainStatus::NeedsRebuild => {
                self.swapchain_needs_rebuild = true;
                return Ok(FrameStatus::SwapchainNeedsRebuild);
            }
        };

        // wait for the previous submission's commands to finish, then restart
        // the command buffer.
        frame_sync.wait_and_restart_command_buffer()?;

        let frame = Frame::new(frame_sync, swapchain_image_index);
        Ok(FrameStatus::FrameAcquired(frame))
    }

    /// Submit a frame's commands to the graphics queue and schedule the
    /// swapchain image for presentation.
    ///
    /// # Params
    ///
    /// * `render_device` - the render device used to create the frames in
    ///   flight.
    /// * `frame` - the frame to present
    pub fn present_frame(&mut self, frame: Frame) -> Result<(), GraphicsError> {
        debug_assert!(frame.frame_index() == self.current_frame);

        let frame_index = frame.frame_index();
        let swapchain_image_index = frame.swapchain_image_index();
        self.frames[frame_index] = Some(frame.take_sync());
        let sync = self.frames[frame_index].as_ref().unwrap();
        let command_buffer = sync.command_pool.primary_command_buffer(0);

        // end the command buffer and submit
        unsafe {
            self.render_device
                .device()
                .end_command_buffer(command_buffer)
                .with_context(|| {
                    format!(
                        "Error ending graphics command buffer for frame {}",
                        self.current_frame
                    )
                })?;

            let command_buffer_infos = [vk::CommandBufferSubmitInfo {
                command_buffer,
                ..Default::default()
            }];
            let wait_infos = [vk::SemaphoreSubmitInfo {
                semaphore: sync.swapchain_image_acquired_semaphore.raw(),
                stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }];
            let signal_infos = [vk::SemaphoreSubmitInfo {
                semaphore: sync.graphics_commands_completed_semaphore.raw(),
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
                sync.graphics_commands_completed_fence.raw(),
            )?;
        }

        unsafe {
            let status = self
                .swapchain()
                .present_swapchain_image(
                    swapchain_image_index,
                    &[sync.graphics_commands_completed_semaphore.raw()],
                )
                .with_context(|| {
                    format!(
                    "Error while presenting swapchain image {} for frame {}",
                    swapchain_image_index, self.current_frame,
                )
                })?;
            if status == SwapchainStatus::NeedsRebuild {
                self.swapchain_needs_rebuild = true;
            }
        };
        Ok(())
    }
}

impl Drop for FramesInFlight {
    /// Destroy all frame resources.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application should drop this before any resources which are used
    ///     by any frames.
    fn drop(&mut self) {
        unsafe {
            self.wait_for_all_frames_to_complete()
                .expect("Error while waiting for frames to complete!");
        }
    }
}

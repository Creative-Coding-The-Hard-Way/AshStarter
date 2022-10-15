mod frame;

use {
    super::vulkan_api::Semaphore,
    crate::graphics::{
        vulkan_api::{
            ImageView, RenderDevice, SemaphorePool, Swapchain, SwapchainStatus,
            VulkanDebug, VulkanError,
        },
        GraphicsError,
    },
    std::sync::Arc,
};

pub use self::frame::Frame;

/// The result of acquiring a swapchain frame.
pub enum AcquiredFrame {
    Available(Frame),
    SwapchainNeedsRebuild,
}

/// This type owns the application's Swapchain and all resources required to
/// synchronize grahpcis command submissions per-frame.
pub struct SwapchainFrames {
    swapchain_needs_rebuild: bool,
    frames: Vec<Option<Frame>>,
    swapchain: Option<Arc<Swapchain>>,
    semaphore_pool: SemaphorePool,
    render_device: Arc<RenderDevice>,
}

impl SwapchainFrames {
    /// Create a new instance of the swapchain and dependent resources.
    ///
    /// The swapchain is initialized into a state where it always needs to be
    /// rebuilt right away. This means that any swapchain-depentent resources
    /// can be placed into the applications rebuild_swapchain_resources method.
    pub fn new(render_device: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let semaphore_pool = SemaphorePool::new(render_device.clone());
        Ok(Self {
            swapchain_needs_rebuild: true,
            frames: vec![],
            swapchain: None,
            semaphore_pool,
            render_device,
        })
    }

    /// Get the current swapchain Arc.
    pub fn swapchain(&self) -> &Arc<Swapchain> {
        self.swapchain.as_ref().unwrap()
    }

    /// Get the number of swapchain images.
    ///
    /// This can change after calls to rebuild_swapchain.
    pub fn swapchain_image_count(&self) -> usize {
        self.frames.len()
    }

    /// Get the ImageView for the swapchain image of the corresponding index.
    pub fn swapchain_image_view(
        &self,
        swapchain_image_index: usize,
    ) -> Result<&Arc<ImageView>, GraphicsError> {
        let image_view = self.frames[swapchain_image_index]
            .as_ref()
            .ok_or(GraphicsError::FrameMissing)?
            .swapchain_image_view();
        Ok(image_view)
    }

    /// Get the next swapchain image and return the relevant frame object.
    /// The frame must be returned by a call to present_frame.
    pub fn acquire_swapchain_frame(
        &mut self,
    ) -> Result<AcquiredFrame, GraphicsError> {
        if self.swapchain_needs_rebuild {
            return Ok(AcquiredFrame::SwapchainNeedsRebuild);
        }

        let acquire_semaphore = self.semaphore_pool.get_semaphore()?;
        let result = self
            .swapchain
            .as_ref()
            .unwrap()
            .acquire_next_swapchain_image(Some(&acquire_semaphore), None)?;

        let mut current_frame = match result {
            SwapchainStatus::NeedsRebuild => {
                return Ok(AcquiredFrame::SwapchainNeedsRebuild);
            }
            SwapchainStatus::ImageAcquired(index) => self.frames[index]
                .take()
                .ok_or(GraphicsError::FrameMissing)?,
        };

        if let Some(semaphore) =
            current_frame.replace_acquire_semaphore(acquire_semaphore)
        {
            semaphore.set_debug_name("scratch semaphore");
            self.semaphore_pool.return_semaphore(semaphore);
        }

        // Prepare the frame for the application's rendering commands.
        current_frame.reset_frame_commands()?;

        Ok(AcquiredFrame::Available(current_frame))
    }

    /// Submit frame commands and tell the swapchain to present it.
    pub fn present_frame(&mut self, frame: Frame) -> Result<(), GraphicsError> {
        self.present_frame_with_signal(frame, &[])
    }

    /// Submit frame commands and tell the swapchain to present it.
    ///
    /// - graphics_complete_signal_semaphores are signalled when the graphics
    ///   commands for this frame have completed executing
    pub fn present_frame_with_signal(
        &mut self,
        mut frame: Frame,
        graphics_complete_signal_semaphores: &[&Semaphore],
    ) -> Result<(), GraphicsError> {
        frame.submit_frame_commands(graphics_complete_signal_semaphores)?;
        self.swapchain.as_ref().unwrap().present_swapchain_image(
            frame.swapchain_image_index(),
            frame.release_semaphore(),
        )?;

        // return the frame to the set of frames
        let index = frame.swapchain_image_index();
        self.frames[index] = Some(frame);

        Ok(())
    }

    /// Force the swapchain and dependent resources to be rebuilt the next time
    /// a frame is acquired.
    pub fn invalidate_swapchain(&mut self) {
        self.swapchain_needs_rebuild = true;
    }

    /// Waits for all frames to finish executing their graphics commands.
    pub fn wait_for_all_frames_to_complete(
        &mut self,
    ) -> Result<(), GraphicsError> {
        for frame in &mut self.frames {
            frame
                .as_mut()
                .ok_or(GraphicsError::FrameMissing)?
                .wait_for_graphics_commands_to_complete()?;
        }
        Ok(())
    }

    /// Rebuild the Swapchain and all per-frame synchronization resources.
    ///
    /// # Errors
    ///
    /// An error will be returned if the old swapchain cannot be reclaimed with
    /// exclusive ownership. This means any resources which have an
    /// Arc<Swapchain> must be dropped prior to calling this method. This is by
    /// design because it forces the app to crash if something is holding
    /// references to the old swapchain.
    pub fn rebuild_swapchain(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<(), GraphicsError> {
        self.wait_for_all_frames_to_complete()?;

        // Drop all per-frame resources. The number of swapchain images and
        // format could change which will require these to be rebuilt anyways.
        self.frames.clear();

        // Try to get exclusive ownership of the old swapchain if it exists.
        // If ownership cannot be taken it means some resource stil has an Arc
        // and something has not been destroyed before rebuilding the swapchain.
        // (this means a bug in the app logic)
        let old_swap = if let Some(swap_arc) = self.swapchain.take() {
            let swap_result = Arc::try_unwrap(swap_arc);
            if swap_result.is_err() {
                return Err(GraphicsError::SwapchainOwnershipIsNotUnique);
            }
            swap_result.ok()
        } else {
            None
        };

        // create a new swapchain
        let (w, h) = framebuffer_size;
        self.swapchain = Some(Arc::new(Swapchain::new(
            self.render_device.clone(),
            (w as u32, h as u32),
            old_swap,
        )?));

        // build per-frame resources for each swapchain image
        let image_count =
            self.swapchain.as_ref().unwrap().swapchain_image_count();
        for index in 0..image_count {
            let frame = Frame::new(
                &self.render_device,
                &mut self.semaphore_pool,
                index as usize,
                self.swapchain.as_ref().unwrap().clone(),
            )?;
            frame.set_debug_name(format!("[Frame {}]", index));
            self.frames.push(Some(frame));
        }

        self.swapchain_needs_rebuild = false;
        Ok(())
    }
}

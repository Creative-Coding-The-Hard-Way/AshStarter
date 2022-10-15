mod graphics_pipeline;
mod render_target;

use {
    super::vulkan_api::Semaphore,
    crate::{
        application::GlfwWindow,
        graphics::{
            vulkan_api::{Framebuffer, ImageView, RenderDevice, RenderPass},
            AcquiredFrame, Frame, SwapchainFrames,
        },
    },
    anyhow::Result,
    ash::vk,
    std::sync::Arc,
};

/// Owns all of the resources needed to render multisampled frames to the
/// screen.
pub struct MSAADisplay {
    extent: vk::Extent2D,
    samples: vk::SampleCountFlags,
    msaa_render_target: Arc<ImageView>,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    swapchain_frames: SwapchainFrames,
    render_device: Arc<RenderDevice>,
}

impl MSAADisplay {
    pub fn new(
        render_device: Arc<RenderDevice>,
        glfw_window: &mut GlfwWindow,
        desired_samples: vk::SampleCountFlags,
    ) -> Result<Self> {
        let samples = Self::pick_max_supported_msaa_count(
            &render_device,
            desired_samples,
        );

        let mut swapchain_frames = SwapchainFrames::new(render_device.clone())?;
        swapchain_frames.rebuild_swapchain(
            glfw_window.window_handle.get_framebuffer_size(),
        )?;

        let (msaa_render_target, render_pass, framebuffers) =
            Self::build_swapchain_resources(
                &render_device,
                &swapchain_frames,
                samples,
            )?;

        Ok(Self {
            extent: swapchain_frames.swapchain().extent(),
            samples,
            msaa_render_target,
            render_pass,
            framebuffers,
            swapchain_frames,
            render_device,
        })
    }

    /// Efficiently get the current Swapchain extent.
    pub fn swapchain_extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// Immediately rebuild the swapchain and all dependent resources.
    /// Waits for all in-flight frames to complete before destroying resources,
    /// so the application can safely destroy any swapchain-dependent resources.
    pub fn rebuild_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        self.swapchain_frames.wait_for_all_frames_to_complete()?;
        self.framebuffers.clear();
        self.swapchain_frames.rebuild_swapchain(framebuffer_size)?;
        //
        let (msaa_render_target, render_pass, framebuffers) =
            Self::build_swapchain_resources(
                &self.render_device,
                &self.swapchain_frames,
                self.samples,
            )?;
        self.msaa_render_target = msaa_render_target;
        self.render_pass = render_pass;
        self.framebuffers = framebuffers;
        self.extent = self.swapchain_frames.swapchain().extent();

        Ok(())
    }

    /// Force the swapchain to be rebuilt  the next time a frame is requested.
    pub fn invalidate_swapchain(&mut self) {
        self.swapchain_frames.invalidate_swapchain();
    }

    /// Request a frame from the swapchain and begin an onscreen MSAA render
    /// pass.
    pub fn begin_frame(&mut self) -> Result<AcquiredFrame> {
        let frame = match self.swapchain_frames.acquire_swapchain_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return Ok(AcquiredFrame::SwapchainNeedsRebuild);
            }
            AcquiredFrame::Available(frame) => frame,
        };
        Ok(AcquiredFrame::Available(frame))
    }

    /// Begin a fullscreen MSAA render pass which targets the current swapchain
    /// framebuffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must ensure the render pass begins at the correct
    ///     place in the command buffer.
    pub unsafe fn begin_render_pass(
        &self,
        frame: &mut Frame,
        clear_color: [f32; 4],
    ) {
        let swapchain_extent = self.swapchain_frames.swapchain().extent();
        let framebuffer = &self.framebuffers[frame.swapchain_image_index()];
        frame.command_buffer().begin_render_pass_inline(
            &self.render_pass,
            framebuffer,
            swapchain_extent,
            clear_color,
        );
    }

    /// Return the frame to the swapchain for command buffer execution and
    /// presentation.
    pub fn end_frame(&mut self, frame: Frame) -> Result<()> {
        self.end_frame_with_signal(frame, &[])
    }

    /// Return the frame to the swapchain for command buffer execution and
    /// presentation.
    pub fn end_frame_with_signal(
        &mut self,
        frame: Frame,
        graphics_complete_signal_semaphores: &[&Semaphore],
    ) -> Result<()> {
        self.swapchain_frames.present_frame_with_signal(
            frame,
            graphics_complete_signal_semaphores,
        )?;
        Ok(())
    }

    pub fn swapchain_image_count(&self) -> usize {
        self.swapchain_frames.swapchain_image_count()
    }
}

impl MSAADisplay {
    fn build_swapchain_resources(
        render_device: &Arc<RenderDevice>,
        swapchain_frames: &SwapchainFrames,
        samples: vk::SampleCountFlags,
    ) -> Result<(Arc<ImageView>, RenderPass, Vec<Framebuffer>)> {
        let msaa_target_image_view =
            Arc::new(render_target::create_msaa_image(
                render_device,
                swapchain_frames,
                samples,
            )?);
        let render_pass = render_target::create_msaa_render_pass(
            render_device.clone(),
            swapchain_frames.swapchain().format(),
            samples,
        )?;

        let mut framebuffers = vec![];
        let extent = swapchain_frames.swapchain().extent();
        for i in 0..swapchain_frames.swapchain_image_count() {
            let framebuffer_image_view =
                swapchain_frames.swapchain_image_view(i)?;
            framebuffers.push(Framebuffer::new(
                render_device.clone(),
                &render_pass,
                &[
                    msaa_target_image_view.clone(),
                    framebuffer_image_view.clone(),
                ],
                extent,
            )?);
        }

        Ok((msaa_target_image_view, render_pass, framebuffers))
    }

    /// Query the device for MSAA support.
    ///
    /// # Returns
    ///
    /// The minimum between the `desired` sample count and the sample count
    /// supported by the device.
    ///
    /// e.g. if the device supports 4xMSAA and 8xMSAA is desired, this method
    /// will return 4xMSAA. Similarly, if the device supports 4xMSAA and 2xMSAA
    /// is desired, then this method will return 2xMSAA.
    fn pick_max_supported_msaa_count(
        render_device: &RenderDevice,
        desired: vk::SampleCountFlags,
    ) -> vk::SampleCountFlags {
        let props = render_device.get_physical_device_properties();
        let supported_samples = props
            .limits
            .framebuffer_depth_sample_counts
            .min(props.limits.framebuffer_color_sample_counts);

        let msaa_count = if supported_samples
            .contains(vk::SampleCountFlags::TYPE_64)
        {
            desired.min(vk::SampleCountFlags::TYPE_64)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_32) {
            desired.min(vk::SampleCountFlags::TYPE_32)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_16) {
            desired.min(vk::SampleCountFlags::TYPE_16)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_8) {
            desired.min(vk::SampleCountFlags::TYPE_8)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_4) {
            desired.min(vk::SampleCountFlags::TYPE_4)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_2) {
            desired.min(vk::SampleCountFlags::TYPE_2)
        } else {
            vk::SampleCountFlags::TYPE_1
        };

        log::debug!("Chosen sample count {:#?}", msaa_count);

        msaa_count
    }
}

use {
    crate::graphics::{
        vulkan_api::{raii, Frame, RenderDevice, Swapchain},
        GraphicsError,
    },
    ash::vk,
    std::sync::Arc,
};

/// A utility for managing a render pass and framebuffers which target a given
/// set of images.
///
/// The color pass is single-sampled and does not have a depth/stencil buffer.
#[derive(Debug)]
pub struct ColorPass {
    extent: vk::Extent2D,
    format: vk::Format,
    render_pass: raii::RenderPass,
    framebuffers: Vec<raii::Framebuffer>,
    _image_views: Vec<raii::ImageView>,
    render_device: Arc<RenderDevice>,
}

// Public API
// ----------

impl ColorPass {
    /// Create a render pass with a single coloro attachment which can target
    /// all of the provided images.
    ///
    /// # Params
    ///
    /// * `render_device` - the render device used to create Vulkan resources
    /// * `images` - the images that can be targeted by this render pass
    /// * `format` - the image format for all provided images
    /// * `extent` - the extent for all provided images
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the framebuffers are only valid while the swapchain exists
    ///  - if the swapchain is rebuilt, the framebuffers should be destroyed and
    ///    rebuilt too
    ///  - the targeted images MUST outlive the ColorPass.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        swapchain: &Swapchain,
    ) -> Result<Self, GraphicsError> {
        let render_pass = Self::create_render_pass(
            render_device.clone(),
            swapchain.image_format(),
        )?;
        let image_views = Self::create_image_views(
            render_device.clone(),
            swapchain.image_format(),
            swapchain.images(),
        )?;

        let framebuffers = Self::create_framebuffers(
            render_device.clone(),
            render_pass.raw(),
            swapchain.extent(),
            &image_views,
        )?;

        Ok(Self {
            extent: swapchain.extent(),
            format: swapchain.image_format(),
            render_pass,
            framebuffers,
            _image_views: image_views,
            render_device,
        })
    }

    /// The current extent.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// The current format.
    pub fn format(&self) -> vk::Format {
        self.format
    }

    /// The current render pass.
    pub fn render_pass(&self) -> &raii::RenderPass {
        &self.render_pass
    }

    /// Begin a render pass for the given image index.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - `image_index` is not checked, it is expected to be between 0 and the
    ///     number of images given when the CololPass was created.
    ///   - the ColorPass must not be destroyed until the command buffer
    ///     finishes executing or is discarded.
    pub unsafe fn begin_render_pass_inline(
        &self,
        frame: &Frame,
        clear_color: [f32; 4],
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass.raw(),
            framebuffer: self.framebuffers[frame.swapchain_image_index()].raw(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent(),
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        self.render_device.device().cmd_begin_render_pass(
            frame.command_buffer(),
            &begin_info,
            vk::SubpassContents::INLINE,
        );
    }
}

// Private API
// -----------

impl ColorPass {
    /// Create image views for each image.
    ///
    /// # Params
    ///
    /// * `device` - the Vulkan device used to create all resources
    /// * `format` - the image format (same for all images)
    /// * `images` - the images to create views for
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - The caller is responsible for destroying the views before the images
    ///    are destroyed.
    ///  - The caller must not destroy the image views while they are in use by
    ///    the GPU.
    unsafe fn create_image_views(
        render_device: Arc<RenderDevice>,
        format: vk::Format,
        images: &[vk::Image],
    ) -> Result<Vec<raii::ImageView>, GraphicsError> {
        let mut image_views = vec![];

        for image in images {
            let image_view = {
                let create_info = vk::ImageViewCreateInfo {
                    image: *image,
                    format,
                    view_type: vk::ImageViewType::TYPE_2D,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                raii::ImageView::new(render_device.clone(), &create_info)?
            };
            image_views.push(image_view);
        }

        Ok(image_views)
    }

    /// Create framebuffers for each image view.
    ///
    /// # Params
    ///
    /// * `device` - the Vulkan device used to create all resources
    /// * `render_pass` - the render pass used to determine render pass
    ///   compatability for the created framebuffers
    /// * `extent` - the size of the targeted images
    /// * `image_views` - the image views to use as framebuffer color
    ///   attachments
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller is responsible for destroying the framebuffers before the
    ///    Vulkan instance
    ///  - the caller must ensure that no pending command buffers still
    ///    reference the framebuffers when they are destroyed
    unsafe fn create_framebuffers(
        render_device: Arc<RenderDevice>,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        image_views: &[raii::ImageView],
    ) -> Result<Vec<raii::Framebuffer>, GraphicsError> {
        let mut framebuffers = vec![];
        let vk::Extent2D { width, height } = extent;
        for image_view in image_views {
            let raw_image_view = image_view.raw();
            let framebuffer = {
                let create_info = vk::FramebufferCreateInfo {
                    render_pass,
                    attachment_count: 1,
                    p_attachments: &raw_image_view,
                    width,
                    height,
                    layers: 1,
                    ..Default::default()
                };
                raii::Framebuffer::new(render_device.clone(), &create_info)?
            };
            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    /// Create a render pass with a single subpass with external dependencies
    /// for input and output.
    ///
    /// # Params
    ///
    /// * `device` - the Vulkan device used to create all resources
    /// * `format` - the targeted image format
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the caller is responsible for destroying the render pass before the
    ///     Vulkan instance
    ///   - access to the renderpass must be externally synchronized.
    unsafe fn create_render_pass(
        render_device: Arc<RenderDevice>,
        format: vk::Format,
    ) -> Result<raii::RenderPass, GraphicsError> {
        let attachments = [
            // The color attachment
            vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                flags: vk::AttachmentDescriptionFlags::empty(),
            },
        ];
        let subpass0_color_attachments = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let subpasses = [vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: subpass0_color_attachments.len() as u32,
            p_color_attachments: subpass0_color_attachments.as_ptr(),
            ..Default::default()
        }];
        // External dependenciesensure that the image layout transitions at the
        // right time because we're using synchronization2 for graphics command
        // submission and the semaphore wait+signal operations both occur at
        // COLOR_ATTACHMENT_OUTPUT to minimize their scope. The subpass
        // dependencies are adjusted to match these signal operations.
        let dependencies = [
            // input dependency
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::NONE,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::empty(),
            },
            // output dependency
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::NONE,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::empty(),
            },
        ];
        let create_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            flags: vk::RenderPassCreateFlags::empty(),
            ..Default::default()
        };
        raii::RenderPass::new(render_device, &create_info)
    }
}

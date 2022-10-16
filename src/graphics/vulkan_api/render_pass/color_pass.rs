use {crate::graphics::GraphicsError, anyhow::Context, ash::vk};

/// A utility for managing a render pass and framebuffers which target a given
/// set of images.
///
/// The color pass is single-sampled and does not have a depth/stencil buffer.
#[derive(Debug)]
pub struct ColorPass {
    extent: vk::Extent2D,
    format: vk::Format,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    image_views: Vec<vk::ImageView>,
}

// Public API
// ----------

impl ColorPass {
    /// Create a render pass with a single coloro attachment which can target
    /// all of the provided images.
    ///
    /// # Params
    ///
    /// * `device` - the render device used to create Vulkan resources
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
        device: &ash::Device,
        images: &[vk::Image],
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Result<Self, GraphicsError> {
        let render_pass = Self::create_render_pass(device, format)?;
        let image_views = Self::create_image_views(device, format, images)?;

        let framebuffers = Self::create_framebuffers(
            device,
            render_pass,
            extent,
            &image_views,
        )?;

        Ok(Self {
            extent,
            format,
            render_pass,
            framebuffers,
            image_views,
        })
    }

    /// Destroy all framebuffers and image views.
    ///
    /// # Params
    ///
    /// * `device` - the render device used to create Vulkan resources
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must ensure that no pending command buffers still
    ///     reference the framebuffers or image views in this object
    ///   - the application must call `destry()` before exit
    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        for &framebuffer in &self.framebuffers {
            device.destroy_framebuffer(framebuffer, None);
        }
        for &image_view in &self.image_views {
            device.destroy_image_view(image_view, None);
        }
        device.destroy_render_pass(self.render_pass, None);
    }

    /// The current extent.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// The current format.
    pub fn format(&self) -> vk::Format {
        self.format
    }

    /// Begin a render pass for the given image index.
    ///
    /// # Params
    ///
    /// * `device` - the render device used to create Vulkan resources
    /// * `command_buffer` - the command buffer to start the subpass in
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - `image_index` is not checked, it is expected to be between 0 and the
    ///     number of images given when the CololPass was created.
    ///   - the ColorPass must not be destroyed until the command buffer
    ///     finishes executing or is discarded.
    pub unsafe fn begin_render_pass(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        subpass_contents: vk::SubpassContents,
        image_index: usize,
        clear_color: [f32; 4],
    ) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[image_index],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent(),
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        device.cmd_begin_render_pass(
            command_buffer,
            &begin_info,
            subpass_contents,
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
    ///  - the caller is responsible for destroying the viwes
    ///  - the caller must ensure that no pending command buffers still
    ///    reference the views when they are destroyed
    unsafe fn create_image_views(
        device: &ash::Device,
        format: vk::Format,
        images: &[vk::Image],
    ) -> Result<Vec<vk::ImageView>, GraphicsError> {
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
                device
                    .create_image_view(&create_info, None)
                    .context("Error creating image view!")?
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
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        image_views: &[vk::ImageView],
    ) -> Result<Vec<vk::Framebuffer>, GraphicsError> {
        let mut framebuffers = vec![];

        let vk::Extent2D { width, height } = extent;
        for image_view in image_views {
            let framebuffer = {
                let create_info = vk::FramebufferCreateInfo {
                    render_pass,
                    attachment_count: 1,
                    p_attachments: image_view,
                    width,
                    height,
                    layers: 1,
                    ..Default::default()
                };
                device
                    .create_framebuffer(&create_info, None)
                    .context("Error creating framebuffer!")?
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
        device: &ash::Device,
        format: vk::Format,
    ) -> Result<vk::RenderPass, GraphicsError> {
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
        let render_pass = unsafe {
            device
                .create_render_pass(&create_info, None)
                .context("Unexpected creating a single pass render pass!")?
        };
        Ok(render_pass)
    }
}

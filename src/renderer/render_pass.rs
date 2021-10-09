use super::{RenderPass, RenderPassArgs};

use crate::vulkan::{
    errors::{RenderDeviceError, VulkanError},
    RenderDevice,
};

use ash::{version::DeviceV1_0, vk};

impl RenderPass {
    /// Create a new render pass wrapper.
    pub fn new(
        vk_dev: &RenderDevice,
        name: impl Into<String>,
        args: RenderPassArgs,
    ) -> Result<Self, VulkanError> {
        let owned_name = name.into();
        let render_pass =
            vk_dev.create_render_pass_configured(&owned_name, &args)?;
        let framebuffers = vk_dev.create_framebuffers(
            &render_pass,
            format!("{} Framebuffer", &owned_name),
        )?;
        Ok(Self {
            args,
            render_pass,
            framebuffers,
            name: owned_name,
        })
    }

    /// Destroy all Vulkan resources.
    ///
    /// # unsafe
    ///
    /// - because the application must ensure the framebuffers and render pass
    ///   are not in-use by the GPU when this method is called.
    pub unsafe fn destroy(&mut self, vk_dev: &RenderDevice) {
        for framebuffer in self.framebuffers.drain(..) {
            vk_dev.logical_device.destroy_framebuffer(framebuffer, None);
        }
        vk_dev
            .logical_device
            .destroy_render_pass(self.render_pass, None);
    }

    /// Called when the swapchain has been rebuilt.
    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> Result<(), VulkanError> {
        self.destroy(vk_dev);
        self.render_pass =
            vk_dev.create_render_pass_configured(&self.name, &self.args)?;
        self.framebuffers = vk_dev.create_framebuffers(
            &self.render_pass,
            format!("{} Framebuffer", self.name),
        )?;
        Ok(())
    }

    /// Begin the render pass for the current image framebuffer.
    pub fn begin_render_pass(
        &self,
        vk_dev: &RenderDevice,
        cmd: vk::CommandBuffer,
        current_image: usize,
    ) {
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[current_image],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk_dev.swapchain().extent,
            },
            clear_value_count: self.args.clear_count(),
            p_clear_values: self.args.clear_colors_ptr(),
            ..Default::default()
        };
        unsafe {
            vk_dev.logical_device.cmd_begin_render_pass(
                cmd,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    /// End the render pass.
    pub fn end_render_pass(
        &self,
        vk_dev: &RenderDevice,
        cmd: vk::CommandBuffer,
    ) {
        unsafe {
            vk_dev.logical_device.cmd_end_render_pass(cmd);
        }
    }
}

impl RenderPassArgs {
    fn initial_layout(&self) -> vk::ImageLayout {
        if self.first {
            vk::ImageLayout::UNDEFINED
        } else {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        }
    }

    fn final_layout(&self) -> vk::ImageLayout {
        if self.last {
            vk::ImageLayout::PRESENT_SRC_KHR
        } else {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        }
    }

    fn load_op(&self) -> vk::AttachmentLoadOp {
        if self.clear_colors.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        }
    }

    fn clear_count(&self) -> u32 {
        self.clear_colors
            .as_ref()
            .map(|colors| colors.len() as u32)
            .unwrap_or(0)
    }

    fn clear_colors_ptr(&self) -> *const vk::ClearValue {
        self.clear_colors
            .as_ref()
            .map(|color| color.as_ptr())
            .unwrap_or(std::ptr::null())
    }
}

impl Default for RenderPassArgs {
    fn default() -> Self {
        RenderPassArgs {
            first: false,
            last: false,
            clear_colors: None,
        }
    }
}

impl RenderDevice {
    /// Create a render bass based on the provided render pass args.
    fn create_render_pass_configured(
        &self,
        name: impl Into<String>,
        args: &RenderPassArgs,
    ) -> Result<vk::RenderPass, RenderDeviceError> {
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: self.swapchain().format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: args.load_op(),
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: args.initial_layout(),
            final_layout: args.final_layout(),
        };
        let color_attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];
        let subpass = vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_reference,
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        };
        let attachments = [color_attachment];
        let render_pass_info = vk::RenderPassCreateInfo {
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: 1,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };
        let render_pass = unsafe {
            self.logical_device
                .create_render_pass(&render_pass_info, None)
                .map_err(RenderDeviceError::UnableToCreateRenderPass)?
        };
        self.name_vulkan_object(
            format!("{} RenderPass", name.into()),
            vk::ObjectType::RENDER_PASS,
            render_pass,
        )?;
        Ok(render_pass)
    }
}

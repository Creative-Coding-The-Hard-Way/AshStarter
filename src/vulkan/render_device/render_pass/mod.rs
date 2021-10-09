use super::{RenderDevice, RenderDeviceError, RenderPassArgs};

use ash::{version::DeviceV1_0, vk};

impl RenderDevice {
    /// Create a new render pass.
    pub fn create_render_pass(
        &self,
        args: RenderPassArgs,
    ) -> Result<vk::RenderPass, RenderDeviceError> {
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: self.swapchain().format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
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

        unsafe {
            self.logical_device
                .create_render_pass(&render_pass_info, None)
                .map_err(RenderDeviceError::UnableToCreateRenderPass)
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
}

impl Default for RenderPassArgs {
    fn default() -> Self {
        RenderPassArgs {
            first: false,
            last: false,
        }
    }
}

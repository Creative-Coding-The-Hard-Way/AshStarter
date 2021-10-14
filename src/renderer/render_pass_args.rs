use super::RenderPassArgs;

use crate::vulkan::{errors::RenderPassError, RenderDevice, RenderPass};

use ::{ash::vk, std::sync::Arc};

impl Default for RenderPassArgs {
    fn default() -> Self {
        RenderPassArgs {
            first: false,
            last: false,
            clear_colors: None,
        }
    }
}

impl RenderPassArgs {
    pub fn clear_count(&self) -> u32 {
        self.clear_colors
            .as_ref()
            .map(|colors| colors.len() as u32)
            .unwrap_or(0)
    }

    pub fn clear_colors_ptr(&self) -> *const vk::ClearValue {
        self.clear_colors
            .as_ref()
            .map(|color| color.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub fn create_render_pass(
        &self,
        vk_dev: Arc<RenderDevice>,
    ) -> Result<RenderPass, RenderPassError> {
        let format = vk_dev.with_swapchain(|swapchain| swapchain.format);
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: self.load_op(),
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: self.initial_layout(),
            final_layout: self.final_layout(),
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
        RenderPass::new(vk_dev, &render_pass_info)
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
}

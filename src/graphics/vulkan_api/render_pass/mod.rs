use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError};

/// An owned Vulkan render pass.
pub struct RenderPass {
    render_pass: vk::RenderPass,
    render_device: Arc<RenderDevice>,
}

impl RenderPass {
    /// Create a single-sampled render pass with the requested format.
    pub fn single_sampled(
        render_device: Arc<RenderDevice>,
        format: vk::Format,
    ) -> Result<Self, VulkanError> {
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
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
        let create_info = vk::RenderPassCreateInfo {
            attachment_count: 1,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        Self::new(render_device, &create_info)
    }

    /// Create a new owned Render Pass with the given create info.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<Self, VulkanError> {
        let render_pass =
            unsafe { render_device.create_render_pass(create_info)? };

        Ok(Self {
            render_pass,
            render_device,
        })
    }

    /// Get the raw Vulkan render pass handle.
    ///
    /// # Safety
    ///
    /// Unsafe because ownership is not transferred. The caller is responsible
    /// for not referencing the returned handle after this object has been
    /// dropped.
    pub unsafe fn raw(&self) -> vk::RenderPass {
        self.render_pass
    }
}

impl VulkanDebug for RenderPass {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::RENDER_PASS,
            self.render_pass,
        )
    }
}

impl Drop for RenderPass {
    /// # Safety
    ///
    /// The application must ensure that the RenderPass is not being used by the
    /// GPU when it is dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_render_pass(self.render_pass);
        }
    }
}

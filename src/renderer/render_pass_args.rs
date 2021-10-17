use super::RenderPassArgs;

use crate::vulkan::{
    errors::{FramebufferError, RenderPassError, VulkanError},
    Framebuffer, Image, ImageView, MemoryAllocator, RenderDevice, RenderPass,
    VulkanDebug,
};

use ::{ash::vk, std::sync::Arc};

impl Default for RenderPassArgs {
    fn default() -> Self {
        Self {
            first: false,
            last: false,
            samples: vk::SampleCountFlags::TYPE_1,
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

    pub fn create_msaa_render_target(
        &self,
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<Arc<ImageView>, VulkanError> {
        let (swap_extent, format) =
            vk_dev.with_swapchain(|swap| (swap.extent, swap.format));
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width: swap_extent.width,
                height: swap_extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            samples: self.samples,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let msaa_render_target = Arc::new(Image::new(
            vk_dev.clone(),
            vk_alloc,
            &create_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?);
        let view = Arc::new(ImageView::new_2d(
            msaa_render_target,
            format,
            vk::ImageAspectFlags::COLOR,
        )?);
        Ok(view)
    }

    /// Create framebuffers which are compatable with the renderpass defined
    /// by these args.
    pub fn create_framebuffers(
        &self,
        vk_dev: Arc<RenderDevice>,
        render_pass: &RenderPass,
        msaa_render_target: &ImageView,
    ) -> Result<Vec<Framebuffer>, FramebufferError> {
        let name = "Framebuffer";
        vk_dev.with_swapchain(
            |swapchain| -> Result<Vec<Framebuffer>, FramebufferError> {
                let mut framebuffers = vec![];
                for i in 0..swapchain.image_views.len() {
                    let views = if self.last {
                        vec![msaa_render_target.raw, swapchain.image_views[i]]
                    } else {
                        vec![msaa_render_target.raw]
                    };
                    let framebuffer = Framebuffer::with_color_attachments(
                        vk_dev.clone(),
                        render_pass.raw,
                        &views,
                        swapchain.extent,
                    )?;
                    framebuffer.set_debug_name(format!("{} - {}", name, i))?;
                    framebuffers.push(framebuffer);
                }
                Ok(framebuffers)
            },
        )
    }

    pub fn create_render_pass(
        &self,
        vk_dev: Arc<RenderDevice>,
    ) -> Result<RenderPass, RenderPassError> {
        let format = vk_dev.with_swapchain(|swapchain| swapchain.format);
        let color_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples: self.samples,
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

        let color_resolve_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };
        let resolve_attachment_reference = vk::AttachmentReference {
            attachment: 1,
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
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
            p_resolve_attachments: if self.last {
                &resolve_attachment_reference
            } else {
                std::ptr::null()
            },
        };
        let attachments = if self.last {
            vec![color_attachment, color_resolve_attachment]
        } else {
            vec![color_attachment]
        };
        let render_pass_info = vk::RenderPassCreateInfo {
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
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
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
    }

    fn load_op(&self) -> vk::AttachmentLoadOp {
        if self.clear_colors.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        }
    }
}

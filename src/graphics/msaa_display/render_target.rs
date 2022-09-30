use std::sync::Arc;

use ash::vk;

use crate::graphics::{
    vulkan_api::{Image, ImageView, RenderDevice, RenderPass, VulkanError},
    SwapchainFrames,
};

pub(super) fn create_msaa_render_pass(
    render_device: Arc<RenderDevice>,
    format: vk::Format,
    samples: vk::SampleCountFlags,
) -> Result<RenderPass, VulkanError> {
    let color_attachments = [
        // msaa color buffer
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        // framebuffer resolve attachment
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        },
    ];
    let color_attachment_reference = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };
    let resolve_attachment_reference = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };
    let subpass = vk::SubpassDescription {
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: std::ptr::null(),
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_reference,
        p_resolve_attachments: &resolve_attachment_reference,
        p_depth_stencil_attachment: std::ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: std::ptr::null(),
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
    let create_info = vk::RenderPassCreateInfo {
        attachment_count: color_attachments.len() as u32,
        p_attachments: color_attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: dependencies.len() as u32,
        p_dependencies: dependencies.as_ptr(),
        ..Default::default()
    };

    RenderPass::new(render_device, &create_info)
}
/// Create the msaa render target image and view.
pub(super) fn create_msaa_image(
    render_device: &Arc<RenderDevice>,
    swapchain_frames: &SwapchainFrames,
    samples: vk::SampleCountFlags,
) -> Result<ImageView, VulkanError> {
    let vk::Extent2D { width, height } = swapchain_frames.swapchain().extent();
    let format = swapchain_frames.swapchain().format();
    let image = {
        let create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples,
            usage: vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            tiling: vk::ImageTiling::OPTIMAL,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        };
        Arc::new(Image::new(render_device.clone(), &create_info)?)
    };
    let create_info = vk::ImageViewCreateInfo {
        image: unsafe { *image.raw() },
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    ImageView::for_image(render_device.clone(), image, &create_info)
}

//! Functions to create a proper render pass for this application's graphics
//! pipeline.

use crate::application::{Device, Swapchain};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Create a render pass for the graphics pipeline.
pub fn create_render_pass(
    device: &Device,
    swapchain: &Swapchain,
) -> Result<vk::RenderPass> {
    let attachments = vec![vk::AttachmentDescription::builder()
        .format(swapchain.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build()];

    let color_references = vec![vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()];

    let subpasses = vec![vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_references)
        .build()];

    let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses);

    let render_pass = unsafe {
        device
            .logical_device
            .create_render_pass(&create_info, None)?
    };

    device.name_vulkan_object(
        "Application Render Pass",
        vk::ObjectType::RENDER_PASS,
        &render_pass,
    )?;

    Ok(render_pass)
}

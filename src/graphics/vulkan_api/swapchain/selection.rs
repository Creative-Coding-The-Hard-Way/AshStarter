use ash::vk;

use crate::{
    graphics::vulkan_api::{RenderDevice, VulkanError},
    logging::PrettyList,
};

pub fn choose_surface_format(
    render_device: &RenderDevice,
) -> vk::SurfaceFormatKHR {
    let formats = render_device.supported_surface_formats();
    log::debug!("Available Surface Formats: {:#?}", PrettyList(&formats));

    let format = formats
        .iter()
        .cloned()
        .find(|format| {
            format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && format.format == vk::Format::R8G8B8A8_SRGB
        })
        .unwrap_or_else(|| formats[0]);

    log::debug!("Chose Surface Format: {:#?}", format);
    format
}

pub fn choose_present_mode(render_device: &RenderDevice) -> vk::PresentModeKHR {
    let modes = render_device.supported_presentation_modes();
    log::debug!("Available Presentation Modes: {:#?}", PrettyList(&modes));

    let mode = if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::IMMEDIATE
    };

    log::debug!("Chose Present Mode: {:#?}", mode);
    mode
}

pub fn choose_swap_extent(
    render_device: &RenderDevice,
    framebuffer_size: (u32, u32),
) -> Result<vk::Extent2D, VulkanError> {
    let capabilities = render_device.surface_capabilities()?;
    if capabilities.current_extent.width != u32::MAX {
        log::debug!(
            "use current swapchain extent {:?}",
            capabilities.current_extent
        );
        Ok(capabilities.current_extent)
    } else {
        let (width, height) = framebuffer_size;
        let extent = vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        };
        log::debug!("use computed extent {:?}", extent);
        Ok(extent)
    }
}

pub fn choose_image_count(
    render_device: &RenderDevice,
) -> Result<u32, VulkanError> {
    let capabilities = render_device.surface_capabilities()?;
    let proposed_image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 {
        Ok(std::cmp::min(
            proposed_image_count,
            capabilities.max_image_count,
        ))
    } else {
        Ok(proposed_image_count)
    }
}

pub fn create_image_views(
    render_device: &RenderDevice,
    swapchain_images: &[vk::Image],
    format: vk::Format,
) -> Result<Vec<vk::ImageView>, VulkanError> {
    let mut image_views = vec![];
    for (i, &image) in swapchain_images.iter().enumerate() {
        let create_info = vk::ImageViewCreateInfo {
            image,
            format,
            view_type: vk::ImageViewType::TYPE_2D,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            ..Default::default()
        };
        let image_view =
            unsafe { render_device.create_image_view(&create_info)? };
        render_device.name_vulkan_object(
            format!("swapchain image view {}", i),
            vk::ObjectType::IMAGE_VIEW,
            image_view,
        );
        image_views.push(image_view);
    }
    Ok(image_views)
}

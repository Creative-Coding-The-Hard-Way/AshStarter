use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use ccthw::graphics::vulkan_api::{
    CommandBuffer, CommandPool, Fence, HostCoherentBuffer, Image, ImageView,
    RenderDevice, Sampler,
};

pub fn load_texture(
    render_device: &Arc<RenderDevice>,
) -> Result<(ImageView, Sampler)> {
    let image = image::load_from_memory_with_format(
        include_bytes!("./assets/example_texture.png"),
        image::ImageFormat::Png,
    )?;
    let (width, height) = (image.width(), image.height());
    let image_bytes = image.into_rgba8().into_raw();

    let mut staging_buffer = HostCoherentBuffer::<u8>::new(
        render_device.clone(),
        vk::BufferUsageFlags::TRANSFER_SRC,
        image_bytes.len(),
    )?;
    unsafe {
        // safe because the staging buffer is not being used
        staging_buffer.as_slice_mut()?.copy_from_slice(&image_bytes);
    }

    let image = {
        let create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            samples: vk::SampleCountFlags::TYPE_1,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        Arc::new(Image::new(render_device.clone(), &create_info)?)
    };

    let command_pool = Arc::new(CommandPool::new(
        render_device.clone(),
        render_device.graphics_queue_family_index(),
        vk::CommandPoolCreateFlags::TRANSIENT,
    )?);
    let command_buffer = CommandBuffer::new(
        render_device.clone(),
        command_pool,
        vk::CommandBufferLevel::PRIMARY,
    )?;

    command_buffer.begin_one_time_submit()?;

    let subresource_range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    unsafe {
        command_buffer
            .pipeline_image_memory_barriers(
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                &[vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    src_queue_family_index: render_device
                        .graphics_queue_family_index(),
                    dst_queue_family_index: render_device
                        .graphics_queue_family_index(),
                    image: *image.raw(),
                    subresource_range,
                    ..Default::default()
                }],
            )
            .copy_buffer_to_image(
                &staging_buffer,
                &image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: subresource_range.aspect_mask,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                    image_extent: vk::Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                }],
            )
            .pipeline_image_memory_barriers(
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                &[vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    src_queue_family_index: render_device
                        .graphics_queue_family_index(),
                    dst_queue_family_index: render_device
                        .graphics_queue_family_index(),
                    image: *image.raw(),
                    subresource_range,
                    ..Default::default()
                }],
            );
    }

    command_buffer.end_command_buffer()?;

    let fence = Fence::new(render_device.clone())?;
    fence.reset()?;
    unsafe {
        command_buffer.submit_graphics_commands(&[], &[], &[], Some(&fence))?;
    };
    fence.wait_and_reset()?;

    // create view
    // -----------

    let image_view = {
        let create_info = vk::ImageViewCreateInfo {
            image: unsafe { *image.raw() },
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            components: vk::ComponentMapping::default(),
            flags: vk::ImageViewCreateFlags::empty(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        ImageView::for_image(render_device.clone(), image, &create_info)?
    };

    // create sampler
    // --------------

    let sampler = {
        let create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            ..Default::default()
        };
        Sampler::new(render_device.clone(), &create_info)?
    };

    Ok((image_view, sampler))
}

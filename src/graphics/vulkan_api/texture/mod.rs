use {
    crate::graphics::{
        vulkan_api::{raii, OneTimeSubmitCommandBuffer, RenderDevice},
        GraphicsError,
    },
    anyhow::Context,
    ash::vk,
    std::{path::Path, sync::Arc},
};

/// Represents a 2D rgba texture which can be used by shaders.
pub struct Texture2D {
    pub image_view: raii::ImageView,
    pub image: raii::Image,
}

pub struct TextureLoader {
    staging_buffer: raii::Buffer,
    one_time_submit: OneTimeSubmitCommandBuffer,
    render_device: Arc<RenderDevice>,
}

impl TextureLoader {
    /// Create a new Texture Loader which can build textures from images on the
    /// harddrive.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - the application must call destroy on this instance before the render
    ///   device is dropped.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
    ) -> Result<Self, GraphicsError> {
        let staging_buffer = Self::allocate_staging_buffer(
            render_device.clone(),
            1024 * 1024 * 4,
        )?;

        let one_time_submit = OneTimeSubmitCommandBuffer::new(
            render_device.clone(),
            render_device.graphics_queue().clone(),
        )?;

        Ok(Self {
            staging_buffer,
            one_time_submit,
            render_device,
        })
    }

    /// Read image data from a file on disk and create a 2D texture.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - the caller is responsible for destroying the returned texture before
    ///   render device is dropped
    pub unsafe fn load_texture_2d(
        &mut self,
        texture_path: impl AsRef<Path>,
    ) -> Result<Texture2D, GraphicsError> {
        let img = image::io::Reader::open(&texture_path)
            .with_context(|| {
                format!(
                    "Unable to read texture image from path {:?}",
                    texture_path.as_ref()
                )
            })?
            .decode()
            .with_context(|| {
                format!(
                    "Unable to decode texture image at {:?}",
                    texture_path.as_ref()
                )
            })?
            .into_rgba8();

        self.resize_staging_buffer(
            self.render_device.clone(),
            (img.as_raw().len() * std::mem::size_of::<u8>()) as u64,
        )?;

        // Write image data into the staging buffer
        unsafe {
            let ptr = self
                .staging_buffer
                .allocation()
                .map(self.render_device.device())?;
            assert!(ptr as usize % std::mem::align_of::<u8>() == 0);
            let data = std::slice::from_raw_parts_mut(
                ptr as *mut u8,
                img.as_raw().len(),
            );
            data.copy_from_slice(img.as_raw());
        };

        let image = unsafe {
            let queue_family_index =
                self.render_device.graphics_queue().family_index();
            let create_info = vk::ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::R8G8B8A8_UNORM,
                mip_levels: 1,
                array_layers: 1,
                initial_layout: vk::ImageLayout::UNDEFINED,
                samples: vk::SampleCountFlags::TYPE_1,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 1,
                p_queue_family_indices: &queue_family_index,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                flags: vk::ImageCreateFlags::empty(),
                extent: vk::Extent3D {
                    width: img.width(),
                    height: img.height(),
                    depth: 1,
                },
                ..vk::ImageCreateInfo::default()
            };
            raii::Image::new(
                self.render_device.clone(),
                &create_info,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?
        };

        let image_view = unsafe {
            let create_info = vk::ImageViewCreateInfo {
                image: image.raw(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: vk::Format::R8G8B8A8_UNORM,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    base_array_layer: 0,
                    base_mip_level: 0,
                },
                ..Default::default()
            };
            raii::ImageView::new(self.render_device.clone(), &create_info)?
        };

        unsafe {
            let image_memory_barrier_before = vk::ImageMemoryBarrier2 {
                src_stage_mask: vk::PipelineStageFlags2::TOP_OF_PIPE,
                src_access_mask: vk::AccessFlags2::NONE,
                dst_stage_mask: vk::PipelineStageFlags2::TRANSFER,
                dst_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                image: image.raw(),
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            let dependency_info_before = vk::DependencyInfo {
                dependency_flags: vk::DependencyFlags::empty(),
                memory_barrier_count: 0,
                buffer_memory_barrier_count: 0,
                image_memory_barrier_count: 1,
                p_image_memory_barriers: &image_memory_barrier_before,
                ..Default::default()
            };
            self.render_device.device().cmd_pipeline_barrier2(
                self.one_time_submit.command_buffer(),
                &dependency_info_before,
            );

            let regions = vk::BufferImageCopy2 {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_offset: vk::Offset3D::default(),
                image_extent: vk::Extent3D {
                    width: img.width(),
                    height: img.height(),
                    depth: 1,
                },
                ..Default::default()
            };
            let copy_buffer_to_image_info2 = vk::CopyBufferToImageInfo2 {
                src_buffer: self.staging_buffer.raw(),
                dst_image: image.raw(),
                dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                region_count: 1,
                p_regions: &regions,
                ..Default::default()
            };
            self.render_device.device().cmd_copy_buffer_to_image2(
                self.one_time_submit.command_buffer(),
                &copy_buffer_to_image_info2,
            );

            let image_memory_barrier_after = vk::ImageMemoryBarrier2 {
                src_stage_mask: vk::PipelineStageFlags2::TRANSFER,
                src_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
                dst_stage_mask: vk::PipelineStageFlags2::FRAGMENT_SHADER,
                dst_access_mask: vk::AccessFlags2::SHADER_SAMPLED_READ,
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image: image.raw(),
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            let dependency_info_after = vk::DependencyInfo {
                dependency_flags: vk::DependencyFlags::empty(),
                memory_barrier_count: 0,
                buffer_memory_barrier_count: 0,
                image_memory_barrier_count: 1,
                p_image_memory_barriers: &image_memory_barrier_after,
                ..Default::default()
            };
            self.render_device.device().cmd_pipeline_barrier2(
                self.one_time_submit.command_buffer(),
                &dependency_info_after,
            );
        };

        // Queue Submission
        self.one_time_submit.sync_submit_and_reset()?;

        Ok(Texture2D { image, image_view })
    }
}

// Private Api
// -----------

impl TextureLoader {
    unsafe fn resize_staging_buffer(
        &mut self,
        render_device: Arc<RenderDevice>,
        size: u64,
    ) -> Result<(), GraphicsError> {
        if self.staging_buffer.allocation().size_in_bytes() > size {
            return Ok(());
        }

        self.staging_buffer =
            Self::allocate_staging_buffer(render_device, size)?;
        Ok(())
    }

    unsafe fn allocate_staging_buffer(
        render_device: Arc<RenderDevice>,
        size: u64,
    ) -> Result<raii::Buffer, GraphicsError> {
        unsafe {
            let index = render_device.graphics_queue().family_index();
            let create_info = vk::BufferCreateInfo {
                size,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 1,
                p_queue_family_indices: &index,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                ..Default::default()
            };
            raii::Buffer::new(
                render_device,
                &create_info,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
        }
    }
}

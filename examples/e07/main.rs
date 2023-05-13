use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            raii, ColorPass, FrameStatus, FramesInFlight,
            OneTimeSubmitCommandBuffer, RenderDevice,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
    std::sync::Arc,
};

mod pipeline;

use self::pipeline::create_pipeline;

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

struct TextureExample {
    frames_in_flight: FramesInFlight,

    // Image resources
    _image: raii::Image,
    _image_view: raii::ImageView,
    _sampler: raii::Sampler,

    // Vertex resources
    _buffer: raii::Buffer,

    // Descriptor Set bindigs
    descriptor_pool: raii::DescriptorPool,
    _descriptor_set_layout: raii::DescriptorSetLayout,

    // Pipeline / Per-Frame resources
    pipeline_layout: raii::PipelineLayout,
    pipeline: raii::Pipeline,
    color_pass: ColorPass,
    render_device: Arc<RenderDevice>,
}

impl State for TextureExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);

        let render_device = unsafe {
            // SAFE because the render device is destroyed when state is
            // dropped.
            let mut device_features = PhysicalDeviceFeatures::default();
            // enable synchronization2 for queue_submit2
            device_features.vulkan_13_features_mut().synchronization2 =
                vk::TRUE;
            window.create_default_render_device(device_features)?
        };

        let frames_in_flight = unsafe {
            // SAFE because the render device is destroyed when state is dropped
            FramesInFlight::new(
                render_device.clone(),
                window.get_framebuffer_size(),
                3,
            )?
        };

        let color_pass = unsafe {
            ColorPass::new(render_device.clone(), frames_in_flight.swapchain())?
        };

        let descriptor_set_layout = unsafe {
            raii::DescriptorSetLayout::new_with_bindings(
                render_device.clone(),
                &[
                    vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX,
                        ..vk::DescriptorSetLayoutBinding::default()
                    },
                    vk::DescriptorSetLayoutBinding {
                        binding: 1,
                        descriptor_type:
                            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                        ..vk::DescriptorSetLayoutBinding::default()
                    },
                ],
            )?
        };
        let pipeline_layout = unsafe {
            raii::PipelineLayout::new_with_layouts_and_ranges(
                render_device.clone(),
                &[descriptor_set_layout.raw()],
                &[],
            )?
        };
        let pipeline = unsafe {
            create_pipeline(
                render_device.clone(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                &pipeline_layout,
                color_pass.render_pass(),
            )?
        };

        let buffer = unsafe {
            let create_info = vk::BufferCreateInfo {
                size: (std::mem::size_of::<Vertex>() * 6) as u64,
                usage: vk::BufferUsageFlags::STORAGE_BUFFER,
                ..vk::BufferCreateInfo::default()
            };
            raii::Buffer::new(
                render_device.clone(),
                &create_info,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?
        };

        let ptr = unsafe { buffer.allocation().map(render_device.device())? };
        let vertices = [
            // top triangle
            Vertex {
                pos: [-0.7, -0.7],
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [0.7, -0.7],
                uv: [1.0, 0.0],
            },
            Vertex {
                pos: [0.7, 0.7],
                uv: [1.0, 1.0],
            },
            // bottom triangle
            Vertex {
                pos: [0.7, 0.7],
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [-0.7, 0.7],
                uv: [0.0, 1.0],
            },
            Vertex {
                pos: [-0.7, -0.7],
                uv: [0.0, 0.0],
            },
        ];
        unsafe {
            std::ptr::write_unaligned(ptr as *mut [Vertex; 6], vertices);
        };

        let img =
            image::io::Reader::open("examples/e07/my_example_texture.png")?
                .decode()?
                .into_rgba8();

        let staging_buffer = unsafe {
            let index = render_device.graphics_queue().family_index();
            let create_info = vk::BufferCreateInfo {
                size: (std::mem::size_of::<u8>() * img.as_raw().len()) as u64,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 1,
                p_queue_family_indices: &index,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                ..Default::default()
            };
            raii::Buffer::new(
                render_device.clone(),
                &create_info,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?
        };

        unsafe {
            let ptr =
                staging_buffer.allocation().map(render_device.device())?;
            assert!(ptr as usize % std::mem::align_of::<u8>() == 0);
            let data = std::slice::from_raw_parts_mut(
                ptr as *mut u8,
                img.as_raw().len(),
            );
            data.copy_from_slice(img.as_raw());
        };

        let image = unsafe {
            let queue_family_index =
                render_device.graphics_queue().family_index();
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
                render_device.clone(),
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
            raii::ImageView::new(render_device.clone(), &create_info)?
        };

        let sampler = unsafe {
            let create_info = vk::SamplerCreateInfo {
                mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                mag_filter: vk::Filter::LINEAR,
                min_filter: vk::Filter::LINEAR,
                ..Default::default()
            };
            raii::Sampler::new(render_device.clone(), &create_info)?
        };

        let mut one_time_submit = unsafe {
            OneTimeSubmitCommandBuffer::new(
                render_device.clone(),
                render_device.graphics_queue().clone(),
            )?
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
            render_device.device().cmd_pipeline_barrier2(
                one_time_submit.command_buffer(),
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
                src_buffer: staging_buffer.raw(),
                dst_image: image.raw(),
                dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                region_count: 1,
                p_regions: &regions,
                ..Default::default()
            };
            render_device.device().cmd_copy_buffer_to_image2(
                one_time_submit.command_buffer(),
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
            render_device.device().cmd_pipeline_barrier2(
                one_time_submit.command_buffer(),
                &dependency_info_after,
            );
        };

        // Queue Submission
        unsafe {
            one_time_submit.sync_submit_and_reset()?;
        };

        let mut descriptor_pool = unsafe {
            raii::DescriptorPool::new_with_sizes(
                render_device.clone(),
                1,
                &[
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                    },
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1,
                    },
                ],
            )?
        };
        let _ = descriptor_pool
            .allocate_descriptor_sets(&[&descriptor_set_layout])?;

        unsafe {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: buffer.raw(),
                offset: 0,
                range: buffer.allocation().size_in_bytes(),
            };
            let image_info = vk::DescriptorImageInfo {
                sampler: sampler.raw(),
                image_view: image_view.raw(),
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };
            render_device.device().update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_pool.descriptor_set(0),
                        dst_binding: 0,
                        dst_array_element: 0,
                        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                        p_buffer_info: &buffer_info,
                        p_image_info: std::ptr::null(),
                        p_texel_buffer_view: std::ptr::null(),
                        ..vk::WriteDescriptorSet::default()
                    },
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_pool.descriptor_set(0),
                        dst_binding: 1,
                        dst_array_element: 0,
                        descriptor_type:
                            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 1,
                        p_image_info: &image_info,
                        ..Default::default()
                    },
                ],
                &[],
            );
        };

        Ok(Self {
            _image: image,
            _image_view: image_view,
            _sampler: sampler,

            _buffer: buffer,
            descriptor_pool,
            _descriptor_set_layout: descriptor_set_layout,
            pipeline_layout,
            pipeline,
            color_pass,
            frames_in_flight,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                window.set_should_close(true);
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, window: &mut GlfwWindow) -> Result<()> {
        let frame = match self.frames_in_flight.acquire_frame()? {
            FrameStatus::FrameAcquired(frame) => frame,
            FrameStatus::SwapchainNeedsRebuild => {
                return self.rebuild_swapchain(window);
            }
        };

        unsafe {
            self.color_pass
                .begin_render_pass_inline(&frame, [0.2, 0.2, 0.3, 1.0]);

            // draw commands go here
            self.render_device.device().cmd_bind_pipeline(
                frame.command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.raw(),
            );
            let vk::Extent2D { width, height } =
                self.frames_in_flight.swapchain().extent();
            self.render_device.device().cmd_set_viewport(
                frame.command_buffer(),
                0,
                &[vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: width as f32,
                    height: height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
            self.render_device.device().cmd_set_scissor(
                frame.command_buffer(),
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D { width, height },
                }],
            );
            self.render_device.device().cmd_bind_descriptor_sets(
                frame.command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.raw(),
                0,
                &[self.descriptor_pool.descriptor_set(0)],
                &[],
            );
            self.render_device.device().cmd_draw(
                frame.command_buffer(),
                6,
                1,
                0,
                0,
            );

            self.render_device
                .device()
                .cmd_end_render_pass(frame.command_buffer());
        }

        self.frames_in_flight.present_frame(frame)?;

        Ok(())
    }
}

impl TextureExample {
    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        unsafe {
            self.frames_in_flight
                .stall_and_rebuild_swapchain(window.get_framebuffer_size())?;

            self.color_pass = ColorPass::new(
                self.render_device.clone(),
                self.frames_in_flight.swapchain(),
            )?;

            self.pipeline = create_pipeline(
                self.render_device.clone(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                &self.pipeline_layout,
                self.color_pass.render_pass(),
            )?;
        };

        Ok(())
    }
}

fn main() -> Result<()> {
    Application::<TextureExample>::run()
}

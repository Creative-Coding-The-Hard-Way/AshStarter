use {
    super::Frame,
    crate::graphics::{
        vulkan_api::{raii, FramesInFlight, RenderDevice, Texture2D},
        GraphicsError,
    },
    ash::vk,
    std::sync::Arc,
};

mod pipeline;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[repr(packed)]
pub struct BindlessVertex {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
    pub pad: [f32; 2],
}

/// A utility for rendering high-performance textured triangles using bindless
/// textures.
pub struct BindlessTriangles {
    texture: Texture2D,

    vertex_count: u32,
    vertex_buffers: Vec<raii::Buffer>,
    vertex_buffer_ptrs: Vec<*mut BindlessVertex>,

    sampler: raii::Sampler,
    descriptor_pool: raii::DescriptorPool,
    _descriptor_set_layout: raii::DescriptorSetLayout,

    pipeline_layout: raii::PipelineLayout,
    pipeline: raii::Pipeline,
    render_device: Arc<RenderDevice>,
}

impl BindlessTriangles {
    /// Create a new instance of bindless triangles.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - This instance must be dropped before the RenderDevice is destroyed.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        render_pass: &raii::RenderPass,
        frames_in_flight: &FramesInFlight,
        texture: Texture2D,
    ) -> Result<Self, GraphicsError> {
        let (descriptor_set_layout, pipeline_layout) =
            pipeline::create_layouts(render_device.clone())?;

        let pipeline = pipeline::create_pipeline(
            render_device.clone(),
            include_bytes!("./shaders/bindless.vert.spv"),
            include_bytes!("./shaders/bindless.frag.spv"),
            &pipeline_layout,
            render_pass,
        )?;

        let descriptor_count = frames_in_flight.frame_count() as u32;
        let mut descriptor_pool = raii::DescriptorPool::new_with_sizes(
            render_device.clone(),
            descriptor_count,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count,
                },
            ],
        )?;
        let layouts = (0..descriptor_count)
            .map(|_| &descriptor_set_layout)
            .collect::<Vec<&raii::DescriptorSetLayout>>();
        let _ = descriptor_pool.allocate_descriptor_sets(&layouts)?;

        let sampler = raii::Sampler::new(
            render_device.clone(),
            &vk::SamplerCreateInfo {
                mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                mag_filter: vk::Filter::LINEAR,
                min_filter: vk::Filter::LINEAR,
                ..Default::default()
            },
        )?;

        let vertex_buffer_count = frames_in_flight.frame_count();
        let mut vertex_buffers = Vec::with_capacity(vertex_buffer_count);
        let mut vertex_buffer_ptrs = Vec::with_capacity(vertex_buffer_count);
        for _ in 0..vertex_buffer_count {
            let (buffer, ptr) =
                Self::allocate_vertex_buffer(&render_device, 1000)?;
            vertex_buffer_ptrs.push(ptr);
            vertex_buffers.push(buffer);
        }

        for (index, vertex_buffer) in vertex_buffers.iter().enumerate() {
            Self::write_descriptor_set(
                &render_device,
                &descriptor_pool,
                index,
                vertex_buffer,
                &texture,
                &sampler,
            );
        }

        Ok(Self {
            texture,
            vertex_count: 0,
            vertex_buffers,
            vertex_buffer_ptrs,
            sampler,
            descriptor_pool,
            _descriptor_set_layout: descriptor_set_layout,
            pipeline_layout,
            pipeline,
            render_device,
        })
    }

    pub fn write_vertices_for_frame(
        &mut self,
        frame: &Frame,
        vertices: &[BindlessVertex],
    ) -> Result<(), GraphicsError> {
        if self.vertex_buffers[frame.frame_index()]
            .allocation()
            .size_in_bytes()
            < std::mem::size_of_val(vertices) as u64
        {
            unsafe {
                self.reallocate_vertex_buffer(frame, vertices.len() as u64)?;
                Self::write_descriptor_set(
                    &self.render_device,
                    &self.descriptor_pool,
                    frame.frame_index(),
                    &self.vertex_buffers[frame.frame_index()],
                    &self.texture,
                    &self.sampler,
                );
            };
        }

        let buffer_data = unsafe {
            std::slice::from_raw_parts_mut(
                self.vertex_buffer_ptrs[frame.frame_index()],
                vertices.len(),
            )
        };
        buffer_data.copy_from_slice(vertices);

        self.vertex_count = vertices.len() as u32;

        Ok(())
    }

    /// Add commands to the frame's command buffer to draw the vertices.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The render pass must already be started.
    pub unsafe fn draw_vertices(
        &self,
        frame: &Frame,
        viewport: vk::Extent2D,
    ) -> Result<(), GraphicsError> {
        self.render_device.device().cmd_bind_pipeline(
            frame.command_buffer(),
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.raw(),
        );

        let vk::Extent2D { width, height } = viewport;
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
            &[self.descriptor_pool.descriptor_set(frame.frame_index())],
            &[],
        );
        self.render_device.device().cmd_draw(
            frame.command_buffer(),
            self.vertex_count,
            1,
            0,
            0,
        );

        Ok(())
    }
}

impl BindlessTriangles {
    /// Reallocate's the current frame's vertex buffer to have capacity for the
    /// requested vertex count.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - A graphics error can render the BindlessTriangles entirely unusable.
    unsafe fn reallocate_vertex_buffer(
        &mut self,
        frame: &Frame,
        vertex_count: u64,
    ) -> Result<(), GraphicsError> {
        let index = frame.frame_index();

        self.vertex_buffers[index]
            .allocation()
            .unmap(self.render_device.device())?;
        self.vertex_buffer_ptrs[index] = std::ptr::null_mut();

        let (buffer, ptr) =
            Self::allocate_vertex_buffer(&self.render_device, vertex_count)?;
        self.vertex_buffers[index] = buffer;
        self.vertex_buffer_ptrs[index] = ptr;

        Ok(())
    }

    /// Allocate a vertex buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not use the associated memory-mapped pointer
    ///     once the buffer has been dropped.
    unsafe fn allocate_vertex_buffer(
        render_device: &Arc<RenderDevice>,
        vertex_count: u64,
    ) -> Result<(raii::Buffer, *mut BindlessVertex), GraphicsError> {
        let queue_family_index = render_device.graphics_queue().family_index();
        let create_info = vk::BufferCreateInfo {
            size: std::mem::size_of::<BindlessVertex>() as u64 * vertex_count,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            queue_family_index_count: 1,
            p_queue_family_indices: &queue_family_index,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = raii::Buffer::new(
            render_device.clone(),
            &create_info,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let ptr = buffer.allocation().map(render_device.device())?;
        debug_assert!(
            ptr as usize % std::mem::align_of::<BindlessVertex>() == 0,
            "CPU Ptr must be align for Vertex data!"
        );

        Ok((buffer, ptr as *mut BindlessVertex))
    }

    /// Write the descriptor set for frame index.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the descriptor set must not be in use by the GPU when it is written.
    unsafe fn write_descriptor_set(
        render_device: &RenderDevice,
        descriptor_pool: &raii::DescriptorPool,
        index: usize,
        vertex_buffer: &raii::Buffer,
        texture: &Texture2D,
        sampler: &raii::Sampler,
    ) {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: vertex_buffer.raw(),
            offset: 0,
            range: vertex_buffer.allocation().size_in_bytes(),
        };
        let image_info = vk::DescriptorImageInfo {
            sampler: sampler.raw(),
            image_view: texture.image_view.raw(),
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };
        render_device.device().update_descriptor_sets(
            &[
                vk::WriteDescriptorSet {
                    dst_set: descriptor_pool.descriptor_set(index),
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
                    dst_set: descriptor_pool.descriptor_set(index),
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_buffer_info: std::ptr::null(),
                    p_image_info: &image_info,
                    p_texel_buffer_view: std::ptr::null(),
                    ..vk::WriteDescriptorSet::default()
                },
            ],
            &[],
        );
    }
}

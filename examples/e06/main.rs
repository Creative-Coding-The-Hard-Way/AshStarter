use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            create_descriptor_set_layout, create_pipeline_layout, ColorPass,
            FrameStatus, FramesInFlight, RenderDevice,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
};

mod pipeline;

use {self::pipeline::create_pipeline, ccthw_ash_allocator::Allocation};

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 4],
}

struct SBOTriangleExample {
    buffer: vk::Buffer,
    allocation: Allocation,
    descriptor_set: vk::DescriptorSet,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    color_pass: ColorPass,
    frames_in_flight: FramesInFlight,
    render_device: RenderDevice,
}

impl State for SBOTriangleExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);

        let mut render_device = unsafe {
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
                &render_device,
                window.get_framebuffer_size(),
                3,
            )?
        };

        let color_pass = unsafe {
            ColorPass::new(
                render_device.device(),
                frames_in_flight.swapchain().images(),
                frames_in_flight.swapchain().image_format(),
                frames_in_flight.swapchain().extent(),
            )?
        };

        let descriptor_set_layout = unsafe {
            create_descriptor_set_layout(
                render_device.device(),
                &[vk::DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    ..vk::DescriptorSetLayoutBinding::default()
                }],
            )?
        };
        let pipeline_layout = unsafe {
            create_pipeline_layout(
                render_device.device(),
                &[descriptor_set_layout],
                &[],
            )?
        };
        let pipeline = unsafe {
            create_pipeline(
                render_device.device(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                pipeline_layout,
                color_pass.render_pass(),
            )?
        };

        let (buffer, allocation) = unsafe {
            let create_info = vk::BufferCreateInfo {
                size: (std::mem::size_of::<Vertex>() * 3) as u64,
                usage: vk::BufferUsageFlags::STORAGE_BUFFER,
                ..vk::BufferCreateInfo::default()
            };
            render_device.memory().allocate_buffer(
                &create_info,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?
        };

        let ptr = unsafe { allocation.map(render_device.device())? };
        let vertices = [
            Vertex {
                pos: [-0.7, -0.7, 0.0, 1.0],
            },
            Vertex {
                pos: [0.0, 0.7, 0.0, 1.0],
            },
            Vertex {
                pos: [0.7, -0.7, 0.0, 1.0],
            },
        ];
        unsafe {
            std::ptr::write_unaligned(ptr as *mut [Vertex; 3], vertices);
        };

        let descriptor_pool = unsafe {
            let pool_sizes = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            }];
            let create_info = vk::DescriptorPoolCreateInfo {
                max_sets: 1,
                pool_size_count: pool_sizes.len() as u32,
                p_pool_sizes: pool_sizes.as_ptr(),
                ..vk::DescriptorPoolCreateInfo::default()
            };
            render_device
                .device()
                .create_descriptor_pool(&create_info, None)?
        };

        let descriptor_set = unsafe {
            let create_info = vk::DescriptorSetAllocateInfo {
                descriptor_pool,
                descriptor_set_count: 1,
                p_set_layouts: &descriptor_set_layout,
                ..vk::DescriptorSetAllocateInfo::default()
            };
            render_device
                .device()
                .allocate_descriptor_sets(&create_info)?[0]
        };

        unsafe {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer,
                offset: allocation.offset_in_bytes(),
                range: allocation.size_in_bytes(),
            };
            render_device.device().update_descriptor_sets(
                &[vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                    p_buffer_info: &buffer_info,
                    p_image_info: std::ptr::null(),
                    p_texel_buffer_view: std::ptr::null(),
                    ..vk::WriteDescriptorSet::default()
                }],
                &[],
            );
        };

        Ok(Self {
            buffer,
            allocation,
            descriptor_set,
            descriptor_pool,
            descriptor_set_layout,
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
        let frame =
            match self.frames_in_flight.acquire_frame(&self.render_device)? {
                FrameStatus::FrameAcquired(frame) => frame,
                FrameStatus::SwapchainNeedsRebuild => {
                    return self.rebuild_swapchain(window);
                }
            };

        unsafe {
            self.color_pass.begin_render_pass(
                self.render_device.device(),
                frame.command_buffer(),
                vk::SubpassContents::INLINE,
                frame.swapchain_image_index(),
                [0.2, 0.2, 0.3, 1.0],
            );

            // draw commands go here
            self.render_device.device().cmd_bind_pipeline(
                frame.command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
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
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );
            self.render_device.device().cmd_draw(
                frame.command_buffer(),
                3,
                1,
                0,
                0,
            );

            self.render_device
                .device()
                .cmd_end_render_pass(frame.command_buffer());
        }

        self.frames_in_flight
            .present_frame(&self.render_device, frame)?;

        Ok(())
    }
}

impl SBOTriangleExample {
    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        unsafe {
            self.frames_in_flight.stall_and_rebuild_swapchain(
                &self.render_device,
                window.get_framebuffer_size(),
            )?;

            self.color_pass.destroy(self.render_device.device());
            self.color_pass = ColorPass::new(
                self.render_device.device(),
                self.frames_in_flight.swapchain().images(),
                self.frames_in_flight.swapchain().image_format(),
                self.frames_in_flight.swapchain().extent(),
            )?;

            self.render_device
                .device()
                .destroy_pipeline(self.pipeline, None);

            self.pipeline = create_pipeline(
                self.render_device.device(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                self.pipeline_layout,
                self.color_pass.render_pass(),
            )?;
        };

        Ok(())
    }
}

impl Drop for SBOTriangleExample {
    fn drop(&mut self) {
        unsafe {
            self.frames_in_flight
                .wait_for_all_frames_to_complete(&self.render_device)
                .expect("Error waiting for all frame operations to complete");

            self.render_device
                .device()
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.render_device
                .memory()
                .free_buffer(self.buffer, self.allocation.clone());

            self.render_device
                .device()
                .destroy_pipeline(self.pipeline, None);
            self.render_device
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.render_device.device().destroy_descriptor_set_layout(
                self.descriptor_set_layout,
                None,
            );
            self.color_pass.destroy(self.render_device.device());
            self.frames_in_flight.destroy(&self.render_device);
            self.render_device.destroy();
        }
    }
}

fn main() -> Result<()> {
    Application::<SBOTriangleExample>::run()
}

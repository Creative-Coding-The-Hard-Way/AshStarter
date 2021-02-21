//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

mod graphics_pipeline;
mod render_context;

pub use self::{
    graphics_pipeline::GraphicsPipeline,
    render_context::{RenderContext, SwapchainState},
};
use crate::rendering::{glfw_window::GlfwWindow, Device, Swapchain};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

pub struct Application {
    window_surface: Arc<GlfwWindow>,
    render_context: RenderContext,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    graphics_pipeline: Arc<GraphicsPipeline>,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let window_surface = GlfwWindow::new(|glfw| {
            let (mut window, event_receiver) = glfw
                .create_window(
                    1366,
                    768,
                    "Ash Starter",
                    glfw::WindowMode::Windowed,
                )
                .context("unable to create the glfw window")?;

            window.set_resizable(true);
            window.set_key_polling(true);
            window.set_size_polling(true);

            Ok((window, event_receiver))
        })?;
        let device = Device::new(window_surface.clone())?;
        let swapchain =
            Swapchain::new(device.clone(), window_surface.clone(), None)?;
        let render_context = RenderContext::new(&device, &swapchain)?;
        let pipeline = GraphicsPipeline::new(&device, &swapchain)?;

        Ok(Self {
            window_surface,
            render_context,
            device: device.clone(),
            swapchain: swapchain.clone(),
            graphics_pipeline: pipeline,
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let events = self
            .window_surface
            .event_receiver
            .borrow_mut()
            .take()
            .unwrap();

        while !self.window_surface.window.borrow().should_close() {
            self.window_surface.glfw.borrow_mut().poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                log::debug!("{:?}", event);
                self.handle_event(event)?;
            }

            let device = &self.device;
            let swapchain = &self.swapchain;
            let graphics_pipeline = &self.graphics_pipeline;
            let status =
                self.render_context.draw_frame(|image_available, frame| {
                    let command_buffer = frame.request_command_buffer()?;
                    Self::record_buffer_commands(
                        device,
                        swapchain,
                        graphics_pipeline,
                        &frame.framebuffer,
                        &command_buffer,
                    )?;
                    frame.submit_command_buffers(
                        image_available,
                        &[command_buffer],
                    )
                })?;
            match status {
                SwapchainState::Ok => {}
                SwapchainState::NeedsRebuild => {
                    self.replace_swapchain()?;
                }
            }
        }
        Ok(())
    }

    /// Update all systems which depend on the swapchain
    fn replace_swapchain(&mut self) -> Result<()> {
        self.swapchain = self.render_context.rebuild_swapchain()?;
        self.graphics_pipeline =
            GraphicsPipeline::new(&self.device, &self.swapchain)?;
        Ok(())
    }

    /// Handle window events and update the application state as needed.
    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        match event {
            glfw::WindowEvent::Key(
                glfw::Key::Escape,
                _,
                glfw::Action::Press,
                _,
            ) => {
                self.window_surface
                    .window
                    .borrow_mut()
                    .set_should_close(true);
            }

            glfw::WindowEvent::FramebufferSize(_, _) => {
                log::info!("resized");
                self.render_context.needs_rebuild();
            }

            _ => {}
        }

        Ok(())
    }

    fn record_buffer_commands(
        device: &Device,
        swapchain: &Swapchain,
        graphics_pipeline: &GraphicsPipeline,
        framebuffer: &vk::Framebuffer,
        command_buffer: &vk::CommandBuffer,
    ) -> Result<()> {
        // begin the command buffer
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());

        // begin the render pass
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(swapchain.render_pass)
            .framebuffer(*framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            // begin the command buffer
            device
                .logical_device
                .begin_command_buffer(*command_buffer, &begin_info)?;

            // begin the render pass
            device.logical_device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            // bind the graphics pipeline
            device.logical_device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.pipeline,
            );

            // draw
            device.logical_device.cmd_draw(
                *command_buffer,
                3, // vertex count
                1, // instance count
                0, // first vertex
                0, // first instance
            );

            // end the render pass
            device.logical_device.cmd_end_render_pass(*command_buffer);

            // end the buffer
            device.logical_device.end_command_buffer(*command_buffer)?;
        }

        Ok(())
    }
}

use std::sync::mpsc::Receiver;

use anyhow::Result;
use ash::vk::{self, Handle};
use glfw::{ClientApiHint, WindowEvent, WindowHint, WindowMode};

use crate::{
    application::ApplicationError,
    graphics::vulkan_api::{Instance, RenderDevice},
};

/// All resources required for running a single-windowed GLFW application which
/// renders graphics using Vulkan.
pub struct GlfwWindow {
    window_pos: (i32, i32),
    window_size: (i32, i32),

    /// The receiver for the Window's events.
    pub event_receiver: Option<Receiver<(f64, WindowEvent)>>,

    /// The GLFW window instance.
    pub window_handle: glfw::Window,

    /// The GLFW library instance.
    pub glfw: glfw::Glfw,
}

impl GlfwWindow {
    /// Create a new GLFW window.
    /// Window hints and configuration can be done after the fact by using the
    /// underlying window handle.
    pub fn new(
        window_title: impl AsRef<str>,
    ) -> Result<Self, ApplicationError> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        if !glfw.vulkan_supported() {
            return Err(ApplicationError::GlfwVulkanNotSupported);
        }

        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::ScaleToMonitor(true));

        let (window_handle, event_receiver) = glfw
            .create_window(
                1366,
                768,
                window_title.as_ref(),
                WindowMode::Windowed,
            )
            .ok_or(ApplicationError::UnableToCreateGLFWWindow)?;

        Ok(Self {
            window_pos: window_handle.get_pos(),
            window_size: window_handle.get_size(),
            event_receiver: Some(event_receiver),
            window_handle,
            glfw,
        })
    }

    pub fn toggle_fullscreen(&mut self) -> Result<(), ApplicationError> {
        let is_fullscreen =
            self.window_handle.with_window_mode(|mode| match mode {
                WindowMode::Windowed => false,
                WindowMode::FullScreen(_) => true,
            });

        if is_fullscreen {
            // Switch to windowed mode.
            let (x, y) = self.window_pos;
            let (w, h) = self.window_size;
            self.window_handle.set_monitor(
                WindowMode::Windowed,
                x,
                y,
                w as u32,
                h as u32,
                None,
            );
        } else {
            // Switch to fullscreen mode.
            // Record the size and position of the non-fullscreen window
            // before switching modes.
            self.window_size = self.window_handle.get_size();
            self.window_pos = self.window_handle.get_pos();
            let window = &mut self.window_handle;
            self.glfw.with_primary_monitor_mut(
                |_, monitor_opt| -> Result<(), ApplicationError> {
                    let monitor = monitor_opt
                        .ok_or(ApplicationError::NoPrimaryMonitor)?;
                    let video_mode = monitor
                        .get_video_mode()
                        .ok_or(ApplicationError::NoPrimaryVideoMode)?;
                    window.set_monitor(
                        WindowMode::FullScreen(monitor),
                        0,
                        0,
                        video_mode.width,
                        video_mode.height,
                        Some(video_mode.refresh_rate),
                    );
                    Ok(())
                },
            )?;
        }
        Ok(())
    }

    pub fn create_render_device(
        &self,
    ) -> Result<RenderDevice, ApplicationError> {
        let instance = self.create_vulkan_instance()?;

        let mut surface_handle: u64 = 0;
        let result =
            vk::Result::from_raw(self.window_handle.create_window_surface(
                unsafe { instance.vulkan_instance_handle().as_raw() as usize },
                std::ptr::null(),
                &mut surface_handle,
            ) as i32);
        if result != vk::Result::SUCCESS {
            return Err(ApplicationError::UnableToCreateSurface(result));
        }

        let render_device = RenderDevice::new(
            instance,
            vk::SurfaceKHR::from_raw(surface_handle),
        )?;

        Ok(render_device)
    }

    fn create_vulkan_instance(&self) -> Result<Instance, ApplicationError> {
        let required_extensions = self
            .glfw
            .get_required_instance_extensions()
            .ok_or(ApplicationError::UnableToGetGLFWInstanceExtensions)?;
        let instance = Instance::new(&required_extensions)?;
        Ok(instance)
    }
}

use {
    crate::graphics::GraphicsError,
    ash::vk,
    ccthw_ash_instance::{
        LogicalDevice, PhysicalDevice, PhysicalDeviceFeatures, VulkanInstance,
    },
    indoc::indoc,
};

mod queue;
mod queue_finder;
mod window_surface;

use {
    self::queue_finder::QueueFinder, ccthw_ash_allocator::MemoryAllocator,
    ccthw_ash_instance::VulkanHandle, window_surface::WindowSurface,
};

pub use self::queue::Queue;

/// A combination of the VulkanInstance, LogicalDevice, and queues required by
/// this application.
#[derive(Debug)]
pub struct RenderDevice {
    graphics_queue: Queue,
    presentation_queue: Queue,
    window_surface: WindowSurface,
    logical_device: LogicalDevice,
    instance: VulkanInstance,
    allocator: MemoryAllocator,
}

// Public Api
// ----------

impl RenderDevice {
    /// Create a new render device.
    ///
    /// # Params
    ///
    /// * `instance` - the VulkanInstance used to create all application
    ///   resources. The RenderDevice takes ownership of the vulkan instance so
    ///   it can be destroyed in the correct order.
    /// * `features` - the physical device features required by this
    ///   application.
    /// * `surface` - the surface this application will use for swapchain
    ///   presentation. Typically provided by the windowing system.
    ///
    /// # Safety
    ///
    /// Unsafe because the application must destroy the render device before
    /// exit. The application must also destroy all resources created by the
    /// logical device before destroying the render device.
    pub unsafe fn new(
        instance: VulkanInstance,
        features: PhysicalDeviceFeatures,
        surface: vk::SurfaceKHR,
    ) -> Result<Self, GraphicsError> {
        let window_surface = WindowSurface::new(&instance, surface);
        let physical_device =
            Self::pick_physical_device(&instance, features, &window_surface)?;
        let queue_finder = QueueFinder::new(&physical_device, &window_surface);
        let logical_device = unsafe {
            // SAFE because the RenderDevice takes ownership of the instance
            // along with the LogicalDevice.
            LogicalDevice::new(
                &instance,
                physical_device.clone(),
                &[ash::extensions::khr::Swapchain::name()
                    .to_owned()
                    .into_string()
                    .unwrap()],
                &queue_finder.queue_family_infos(),
            )?
        };
        let (graphics_queue, presentation_queue) =
            queue_finder.get_queues_from_device(&logical_device);

        let allocator = ccthw_ash_allocator::create_system_allocator(
            instance.ash(),
            logical_device.raw().clone(),
            *physical_device.raw(),
        );

        let render_device = Self {
            graphics_queue,
            presentation_queue,
            window_surface,
            logical_device,
            instance,
            allocator,
        };
        render_device.set_debug_name(
            *render_device.presentation_queue().raw(),
            vk::ObjectType::QUEUE,
            "presentation queue",
        );
        render_device.set_debug_name(
            *render_device.graphics_queue.raw(),
            vk::ObjectType::QUEUE,
            "graphics queue",
        );

        Ok(render_device)
    }

    /// Borrow the device memory allocator.
    pub fn memory(&mut self) -> &mut MemoryAllocator {
        &mut self.allocator
    }

    /// Set the name that shows up in Vulkan debug logs for a given resource.
    ///
    /// # Params
    ///
    /// * `handle` - a Vulkan resource represented by the Ash library
    /// * `object_type` - the Vulkan object type
    /// * `name` - a human-readable name for the object. This will show up in
    ///   debug logs if the object is referenced.
    #[cfg(debug_assertions)]
    pub fn set_debug_name(
        &self,
        handle: impl ash::vk::Handle,
        object_type: vk::ObjectType,
        name: impl Into<String>,
    ) {
        let owned_name = name.into();
        let c_name = std::ffi::CString::new(owned_name).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type,
            object_handle: handle.as_raw(),
            p_object_name: c_name.as_ptr(),
            ..Default::default()
        };
        self.instance.debug_utils_set_object_name(
            unsafe { self.logical_device.raw() },
            &name_info,
        );
    }

    /// Set the name that shows up in Vulkan debug logs for a given resource.
    ///
    /// # Params
    ///
    /// * `handle` - a Vulkan resource represented by the Ash library
    /// * `object_type` - the Vulkan object type
    /// * `name` - a human-readable name for the object. This will show up in
    ///   debug logs if the object is referenced.
    #[cfg(not(debug_assertions))]
    pub fn set_debug_name(
        &self,
        _handle: impl ash::vk::Handle,
        _object_type: vk::ObjectType,
        _name: impl Into<String>,
    ) {
        // no-op on release builds
    }

    /// The queue this application uses for graphics operations.
    pub fn presentation_queue(&self) -> &Queue {
        &self.presentation_queue
    }

    /// The queue this application uses for graphics operations.
    pub fn graphics_queue(&self) -> &Queue {
        &self.graphics_queue
    }

    /// Destroy the logical device and ash instance.
    ///
    /// # Safety
    ///
    /// The application must call this prior to exit. All resources created
    /// using the logical device must be destroyed prior to calling this method.
    /// The application is responsible for synchronizing access to GPU
    /// resources.
    pub unsafe fn destroy(&mut self) {
        self.window_surface.destroy();
    }

    /// The Ash entry used by this RenderDevice.
    pub fn entry(&self) -> &ash::Entry {
        self.instance.entry()
    }

    /// The Ash instance used by this RenderDevice.
    pub fn ash(&self) -> &ash::Instance {
        self.instance.ash()
    }

    /// The Ash logical device used to interface with the underlying Vulkan
    /// hardware device.
    ///
    /// # Safety
    ///
    /// The caller must not keep copies of the device handle after any calls
    /// to `destroy`.
    pub unsafe fn device(&self) -> &ash::Device {
        self.logical_device.raw()
    }

    /// The KHR surface provided by the window system for rendering.
    ///
    /// # Safety
    ///
    /// The caller must not keep copies of the device handle after any calls
    /// to `destroy`.
    pub unsafe fn surface(&self) -> &vk::SurfaceKHR {
        self.window_surface.raw()
    }

    /// Get all of the surface formats supported by this device.
    pub fn get_surface_formats(
        &self,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, GraphicsError> {
        unsafe {
            // Safe because the physical device is checked for support when
            // the Render Device is constructed.
            self.window_surface.get_physical_device_surface_formats(
                self.logical_device.physical_device(),
            )
        }
    }

    /// Get all of the presentation modes supported by this device.
    pub fn get_present_modes(
        &self,
    ) -> Result<Vec<vk::PresentModeKHR>, GraphicsError> {
        unsafe {
            // Safe because the physical device is checked for support when
            // the Render Device is constructed.
            self.window_surface
                .get_physical_device_surface_present_modes(
                    self.logical_device.physical_device(),
                )
        }
    }

    /// Get the surface capabilities for this device.
    pub fn get_surface_capabilities(
        &self,
    ) -> Result<vk::SurfaceCapabilitiesKHR, GraphicsError> {
        unsafe {
            // Safe because the physical device is checked for support when
            // the Render Device is constructed.
            self.window_surface
                .get_surface_capabilities(self.logical_device.physical_device())
        }
    }
}

impl std::fmt::Display for RenderDevice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!(
            indoc!(
                "
                RenderDevice Overview

                {}

                {}

                Graphics {}

                Presentation {}"
            ),
            self.instance,
            self.logical_device,
            self.graphics_queue(),
            self.presentation_queue()
        ))
    }
}

// Private API
// -----------

impl RenderDevice {
    /// Pick a physical device which is suitable for this application.
    ///
    /// # Params
    ///
    /// * `instance` - the Vulkan instance used to access devices on this
    ///   platform.
    /// * `features` - all features required by this application.
    fn pick_physical_device(
        instance: &VulkanInstance,
        features: PhysicalDeviceFeatures,
        window_surface: &WindowSurface,
    ) -> Result<PhysicalDevice, GraphicsError> {
        let devices: Vec<PhysicalDevice> =
            PhysicalDevice::enumerate_supported_devices(instance, &features)?
                .into_iter()
                .filter(|device| {
                    QueueFinder::device_has_required_queues(
                        device,
                        window_surface,
                    )
                })
                .filter(|device| {
                    device.available_extension_names().contains(
                        &ash::extensions::khr::Swapchain::name()
                            .to_owned()
                            .into_string()
                            .unwrap(),
                    )
                })
                .collect();
        let find_device_type =
            |device_type: vk::PhysicalDeviceType| -> Option<&PhysicalDevice> {
                devices.iter().find(|device| {
                    device.properties().properties().device_type == device_type
                })
            };

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::DISCRETE_GPU)
        {
            return Ok(device.clone());
        }

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::INTEGRATED_GPU)
        {
            return Ok(device.clone());
        }

        let device = devices
            .first()
            .ok_or(GraphicsError::NoSuitablePhysicalDevice)?;
        Ok(device.clone())
    }
}

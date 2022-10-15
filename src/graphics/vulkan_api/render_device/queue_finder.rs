use {
    super::WindowSurface,
    crate::graphics::vulkan_api::Queue,
    ash::vk,
    ccthw_ash_instance::{LogicalDevice, PhysicalDevice, QueueFamilyInfo},
    indoc::indoc,
    std::collections::HashMap,
};

/// A helper for picking and creating the device queues for the render device.
pub struct QueueFinder {
    graphics_queue_family_index: usize,
    presentation_queue_family_index: usize,
    families: HashMap<usize, QueueFamilyInfo>,
}

// Public API
// ----------

impl QueueFinder {
    /// Check that a physical device has all of the queues required by this
    /// application.
    ///
    /// # Params
    ///
    /// * `device` - The physical device to consider.
    /// * `window_surface` - the window surface which will be used for
    ///   presenting swapchain images.
    pub fn device_has_required_queues(
        device: &PhysicalDevice,
        window_surface: &WindowSurface,
    ) -> bool {
        let has_graphics_queue =
            Self::find_graphics_queue_family_index(device).is_some();
        let has_present_queue =
            Self::find_presentation_queue_family_index(window_surface, device)
                .is_some();

        has_graphics_queue && has_present_queue
    }

    /// Identify all of the queue family indices for queues required by the
    /// application.
    ///
    /// # Params
    ///
    /// * `device` - The physical device to use when finding queue indices.
    pub fn new(
        device: &PhysicalDevice,
        window_surface: &WindowSurface,
    ) -> Self {
        let graphics_queue_family_index =
            Self::find_graphics_queue_family_index(device).unwrap();
        let presentation_queue_family_index =
            Self::find_presentation_queue_family_index(window_surface, device)
                .unwrap();

        let mut families = HashMap::<usize, QueueFamilyInfo>::new();
        families
            .entry(graphics_queue_family_index)
            .or_insert_with_key(|&index| QueueFamilyInfo::new(index as u32))
            .add_queue_priority(1.0);
        families
            .entry(presentation_queue_family_index)
            .or_insert_with_key(|&index| QueueFamilyInfo::new(index as u32))
            .add_queue_priority(1.0);

        Self {
            graphics_queue_family_index,
            presentation_queue_family_index,
            families,
        }
    }

    /// Get Queue instances for each queue required by this application.
    ///
    /// # Param
    ///
    /// * `logical_device` - the device to get the actual Vulkan queues from.
    ///
    /// # Returns
    ///
    /// A tuple of `(graphics_queue, presentation_queue)`.
    pub fn get_queues_from_device(
        &self,
        logical_device: &LogicalDevice,
    ) -> (Queue, Queue) {
        let mut current_indices = HashMap::<usize, usize>::new();
        let mut next_index = |family_index| {
            let index_ref = current_indices.entry(family_index).or_insert(0);
            let index = *index_ref;
            *index_ref = index + 1;
            index
        };

        let graphics_queue = Queue::new(
            logical_device,
            self.graphics_queue_family_index,
            next_index(self.graphics_queue_family_index),
        );
        let presentation_queue = Queue::new(
            logical_device,
            self.presentation_queue_family_index,
            next_index(self.presentation_queue_family_index),
        );

        (graphics_queue, presentation_queue)
    }

    /// Get the QueueFamilyInfos required for creating a logical device with all
    /// of the rquired queues.
    pub fn queue_family_infos(&self) -> Vec<QueueFamilyInfo> {
        self.families.values().cloned().collect()
    }
}

// Private API
// -----------

impl QueueFinder {
    /// Find a queue on the physical device which supports executing graphics
    /// commands.
    ///
    /// # Params
    ///
    /// * `device` - the physical device to check for support
    ///
    /// # Returns
    ///
    /// The queue family index which support graphics commands.
    fn find_graphics_queue_family_index(
        device: &PhysicalDevice,
    ) -> Option<usize> {
        device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_queue_family_index, props)| {
                props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(queue_family_index, _)| queue_family_index)
    }

    /// Find a queue on on the physical device which supports presenting
    /// swapchain images to the window.
    ///
    /// # Params
    ///
    /// * `device` - the physical device to check for support
    ///
    /// # Returns
    ///
    /// The queue family index for a queue which can present swapchain images
    /// to the window surface.
    pub fn find_presentation_queue_family_index(
        window_surface: &WindowSurface,
        device: &PhysicalDevice,
    ) -> Option<usize> {
        device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(queue_family_index, props)| {
                let result = unsafe {
                    window_surface.get_physical_device_surface_support(
                        device,
                        *queue_family_index,
                    )
                };
                if let Err(err) = result {
                    log::warn!(
                        indoc!(
                            "
                            Error checking for surface support
                              - device {}
                              - queue {} [{:?}]
                              - error {:?}"
                        ),
                        device,
                        queue_family_index,
                        props.queue_flags,
                        err,
                    );
                    false
                } else {
                    result.unwrap()
                }
            })
            .map(|(queue_family_index, _properties)| queue_family_index)
    }
}

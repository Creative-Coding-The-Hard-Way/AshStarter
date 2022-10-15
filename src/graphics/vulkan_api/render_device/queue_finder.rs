use {
    crate::graphics::vulkan_api::Queue,
    ash::vk,
    ccthw_ash_instance::{LogicalDevice, PhysicalDevice, QueueFamilyInfo},
    std::collections::HashMap,
};

/// A helper for picking and creating the device queues for the render device.
pub struct QueueFinder {
    graphics_queue_family_index: usize,
    families: HashMap<usize, QueueFamilyInfo>,
}

impl QueueFinder {
    /// Check that a physical device has all of the queues required by this
    /// application.
    ///
    /// # Params
    ///
    /// * `device` - The physical device to consider.
    pub fn device_has_required_queues(device: &PhysicalDevice) -> bool {
        // only take devices which have the queues we're interested
        // in
        device.queue_family_properties().iter().any(|queue_props| {
            queue_props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
    }

    /// Identify all of the queue family indices for queues required by the
    /// application.
    ///
    /// # Params
    ///
    /// * `device` - The physical device to use when finding queue indices.
    pub fn new(device: &PhysicalDevice) -> Self {
        let graphics_queue_family_index = {
            let mut graphics_index = None;

            for (family_index, family_props) in
                device.queue_family_properties().iter().enumerate()
            {
                if family_props.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_index = Some(family_index);
                }
            }

            // It's fine to unwrap because device_has_required_queues checks
            // that all of the requried queue families are available.
            graphics_index.unwrap()
        };

        let mut families = HashMap::<usize, QueueFamilyInfo>::new();
        families
            .entry(graphics_queue_family_index)
            .or_insert_with_key(|&index| QueueFamilyInfo::new(index as u32))
            .add_queue_priority(1.0);

        Self {
            graphics_queue_family_index,
            families,
        }
    }

    /// Get Queue instances for each queue required by this application.
    pub fn get_queues_from_device(
        &self,
        logical_device: &LogicalDevice,
    ) -> Queue {
        let mut current_indices = HashMap::<usize, usize>::new();
        let mut next_index = |family_index| {
            let index_ref = current_indices.entry(family_index).or_insert(0);
            let index = *index_ref;
            *index_ref = index + 1;
            index
        };

        Queue::new(
            logical_device,
            self.graphics_queue_family_index,
            next_index(self.graphics_queue_family_index),
        )
    }

    /// Get the QueueFamilyInfos required for creating a logical device with all
    /// of the rquired queues.
    pub fn queue_family_infos(&self) -> Vec<QueueFamilyInfo> {
        self.families.values().cloned().collect()
    }
}

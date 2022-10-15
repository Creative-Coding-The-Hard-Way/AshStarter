use {
    crate::graphics::vulkan_api::{
        self,
        render_device::{DeviceQueue, WindowSurface},
        VulkanError,
    },
    ash::vk,
    indoc::indoc,
    std::collections::HashMap,
};

#[derive(Clone, Debug, Default)]
struct PerFamily {
    current_index: u32,
    priorities: Vec<f32>,
}

impl PerFamily {
    fn add_queue(&mut self) {
        self.priorities.push(1.0);
    }

    fn take_queue(&mut self) -> u32 {
        let index = self.current_index;
        self.current_index += 1;
        index
    }
}

/// The indices for all of the required queue families for this application.
pub struct QueueFamilies {
    graphics_family_index: u32,
    present_family_index: u32,
    compute_family_index: u32,
    families: HashMap<u32, PerFamily>,
}

impl QueueFamilies {
    /// Find the queue family indexes for the queues this application needs.
    pub fn find_for_physical_device(
        instance: &vulkan_api::Instance,
        window_surface: &WindowSurface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Self, VulkanError> {
        let queue_family_properties = instance
            .get_physical_device_queue_family_properties(physical_device);

        let mut graphics_family = None;
        let mut present_family = None;
        let mut compute_family = None;

        queue_family_properties
            .iter()
            .enumerate()
            .for_each(|(i, family)| {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_family = Some(i as u32);
                } else if family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                    // attempt to find a dedicated async compute family
                    compute_family = Some(i as u32);
                }

                let present_support = unsafe {
                    window_surface.get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                    )
                };
                match present_support {
                    Ok(true) => {
                        present_family = Some(i as u32);
                    }
                    Err(ref error) => {
                        // This is not necessarily a problem - there could be
                        // other queues to check - but
                        // it's good to know if it's
                        // happening.
                        log::warn!(
                        "Error while checking surface support for device: {:?}",
                        error
                    );
                    }
                    _ => {}
                }
            });

        let graphics_family_index =
            graphics_family.ok_or(VulkanError::UnableToFindGraphicsQueue)?;

        // fall back to a sync compute family if an async compute family could
        // not be found
        let compute_family_index =
            compute_family.unwrap_or(graphics_family_index);

        let present_family_index =
            present_family.ok_or(VulkanError::UnableToFindPresentQueue)?;

        log::debug!(
            indoc! {"
            chosen device queue families:

            - graphics family {} | {:#?}

            - compute family  {} | {:#?}

            - present family  {} | {:#?}
            "},
            graphics_family_index,
            queue_family_properties[graphics_family_index as usize],
            compute_family_index,
            queue_family_properties[compute_family_index as usize],
            present_family_index,
            queue_family_properties[present_family_index as usize],
        );

        let mut families = HashMap::<u32, PerFamily>::new();
        families
            .entry(graphics_family_index)
            .or_default()
            .add_queue();
        families
            .entry(present_family_index)
            .or_default()
            .add_queue();
        families
            .entry(compute_family_index)
            .or_default()
            .add_queue();

        Ok(Self {
            graphics_family_index,
            present_family_index,
            compute_family_index,
            families,
        })
    }

    /// Create a vector of queue create infos.
    /// Automatically handles the situation where the graphis and present queue
    /// are the same.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the DeviceQueueCreateInfo structs contain pointers back to data
    ///     contained within this QueueFamilies instance, so this instance must
    ///     outlive their usage.
    pub unsafe fn as_queue_create_infos(
        &self,
    ) -> Vec<vk::DeviceQueueCreateInfo> {
        self.families
            .iter()
            .map(|(&queue_family_index, per_family)| {
                vk::DeviceQueueCreateInfo {
                    queue_family_index,
                    p_queue_priorities: per_family.priorities.as_ptr(),
                    queue_count: per_family.priorities.len() as u32,
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Get the graphics and present queues from the logical device.
    pub fn get_queues(
        mut self,
        logical_device: &ash::Device,
    ) -> (DeviceQueue, DeviceQueue, DeviceQueue) {
        (
            self.take_queue(logical_device, self.graphics_family_index),
            self.take_queue(logical_device, self.present_family_index),
            self.take_queue(logical_device, self.compute_family_index),
        )
    }
}

impl QueueFamilies {
    /// Get the raw Vulkan queue from the logical device.
    ///
    /// Uses the families map to keep track of which index within the queue
    /// family is being used.
    fn take_queue(
        &mut self,
        logical_device: &ash::Device,
        family_index: u32,
    ) -> DeviceQueue {
        let index = self.families.get_mut(&family_index).unwrap().take_queue();
        let raw_queue =
            unsafe { logical_device.get_device_queue(family_index, index) };
        DeviceQueue::from_raw(raw_queue, family_index, index)
    }
}

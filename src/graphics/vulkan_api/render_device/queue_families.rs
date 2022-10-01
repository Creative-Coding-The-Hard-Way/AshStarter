use ash::vk;

use crate::graphics::vulkan_api::{
    self,
    render_device::{DeviceQueue, WindowSurface},
    VulkanError,
};

const SINGLE_QUEUE_PRIORITY: [f32; 1] = [1.0];
const TWO_QUEUE_PRIORITY: [f32; 2] = [1.0, 1.0];

/// The indices for all of the required queue families for this application.
pub struct QueueFamilies {
    graphics_family_index: u32,
    present_family_index: u32,
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

        queue_family_properties
            .iter()
            .enumerate()
            .for_each(|(i, family)| {
                if family.queue_flags.contains(
                    vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
                ) {
                    graphics_family = Some(i as u32);
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
                        // This is not necessarily a problem - there could be other
                        // queues to check - but it's good to know if it's
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

        let present_family_index =
            present_family.ok_or(VulkanError::UnableToFindPresentQueue)?;

        Ok(Self {
            graphics_family_index,
            present_family_index,
        })
    }

    /// Create a vector of queue create infos.
    /// Automatically handles the situation where the graphis and present queue
    /// are the same.
    pub fn as_queue_create_infos(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut create_infos = vec![vk::DeviceQueueCreateInfo {
            queue_family_index: self.graphics_family_index,
            p_queue_priorities: TWO_QUEUE_PRIORITY.as_ptr(),
            queue_count: 2,
            ..Default::default()
        }];

        // if the queue families are not the same
        if self.graphics_family_index != self.present_family_index {
            create_infos.push(vk::DeviceQueueCreateInfo {
                queue_family_index: self.present_family_index,
                p_queue_priorities: SINGLE_QUEUE_PRIORITY.as_ptr(),
                queue_count: 1,
                ..Default::default()
            });
        }

        create_infos
    }

    /// Get the graphics and present queues from the logical device.
    pub fn get_queues(
        &self,
        logical_device: &ash::Device,
    ) -> (DeviceQueue, DeviceQueue, DeviceQueue) {
        let raw_graphics_queue = unsafe {
            logical_device.get_device_queue(self.graphics_family_index, 0)
        };
        let graphics_queue = DeviceQueue::from_raw(
            raw_graphics_queue,
            self.graphics_family_index,
            0,
        );

        let raw_compute_queue = unsafe {
            logical_device.get_device_queue(self.graphics_family_index, 1)
        };
        let compute_queue = DeviceQueue::from_raw(
            raw_compute_queue,
            self.graphics_family_index,
            1,
        );

        let is_same_family =
            self.graphics_family_index == self.present_family_index;
        let present_queue = if is_same_family {
            graphics_queue
        } else {
            let raw_present_queue = unsafe {
                logical_device.get_device_queue(self.present_family_index, 0)
            };
            DeviceQueue::from_raw(
                raw_present_queue,
                self.present_family_index,
                0,
            )
        };

        (graphics_queue, present_queue, compute_queue)
    }
}

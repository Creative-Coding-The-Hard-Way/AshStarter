use {
    ash::vk,
    ccthw_ash_instance::{LogicalDevice, VulkanHandle},
};

/// A Vulkan device queue.
///
/// The raw Ash Queue handle can be acquired through the VulkanHandle trait.
#[derive(Debug)]
pub struct Queue {
    properties: vk::QueueFamilyProperties,
    family_index: u32,
    index: u32,
    queue: vk::Queue,
}

// Public API
// ----------

impl Queue {
    /// The queue family flags.
    pub fn family_flags(&self) -> vk::QueueFlags {
        self.properties.queue_flags
    }

    /// The queue family index for this queue.
    pub fn family_index(&self) -> u32 {
        self.family_index
    }
}

impl std::fmt::Display for Queue {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format.write_fmt(format_args!(
            "Queue {}:{} - {:?}",
            self.family_index, self.index, self.properties.queue_flags,
        ))
    }
}

impl VulkanHandle for Queue {
    type Handle = vk::Queue;

    unsafe fn raw(&self) -> &Self::Handle {
        &self.queue
    }
}

// Private API
// -----------

impl Queue {
    pub(super) fn new(
        logical_device: &LogicalDevice,
        family_index: usize,
        index: usize,
    ) -> Self {
        let queue = unsafe {
            logical_device
                .raw()
                .get_device_queue(family_index as u32, index as u32)
        };
        let properties = logical_device
            .physical_device()
            .queue_family_properties()[family_index];
        Self {
            properties,
            family_index: family_index as u32,
            index: index as u32,
            queue,
        }
    }
}

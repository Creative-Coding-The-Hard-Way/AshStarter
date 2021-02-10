//! This module provides a structure for finding queue families which support
//! this application.

use anyhow::{Context, Result};
use ash::{version::InstanceV1_0, vk};

/// This struct holds all of the queue indices required by this application.
pub struct QueueFamilyIndices {
    /// the index for the graphics queue
    graphics_family_index: usize,
}

impl QueueFamilyIndices {
    /// Find all of the queue families required by this application.
    ///
    /// Yields an Err if any of the queues cannot be found.
    ///
    /// The implementation is greedy, e.g. the same queue will be used for
    /// multiple operations where possible.
    pub fn find(
        physical_device: &vk::PhysicalDevice,
        ash: &ash::Instance,
    ) -> Result<Self> {
        let queue_families = unsafe {
            ash.get_physical_device_queue_family_properties(*physical_device)
        };

        let mut graphics_family = None;

        queue_families.iter().enumerate().for_each(|(i, family)| {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i);
            }
        });

        let graphics_family_index = graphics_family
            .context("unable to find queue family which supports graphics")?;

        Ok(Self {
            graphics_family_index,
        })
    }
}

use crate::application::Device;

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

pub struct Frame {
    command_pool: vk::CommandPool,
    device: Arc<Device>,
}

impl Frame {
    pub fn new(device: &Arc<Device>) -> Result<Arc<Self>> {
        let command_pool = create_command_pool(device)?;
        Ok(Arc::new(Self {
            command_pool,
            device: device.clone(),
        }))
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

fn create_command_pool(device: &Device) -> Result<vk::CommandPool> {
    let create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(device.graphics_queue.family_id);
    let command_pool = unsafe {
        device
            .logical_device
            .create_command_pool(&create_info, None)
            .context("unable to create the command pool")?
    };
    device.name_vulkan_object(
        "Graphics Command Pool",
        vk::ObjectType::COMMAND_POOL,
        &command_pool,
    )?;
    Ok(command_pool)
}

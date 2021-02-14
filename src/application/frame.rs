use crate::application::{Device, Swapchain};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

pub struct Frame {
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl Frame {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> Result<Arc<Self>> {
        let command_pool = create_command_pool(device)?;
        let command_buffers =
            create_command_buffers(device, swapchain, &command_pool)?;
        Ok(Arc::new(Self {
            command_pool,
            command_buffers,
            swapchain: swapchain.clone(),
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

/// Create the command buffer pool.
///
/// The caller is responsible for destroying the pool before the device.
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

/// Create one command buffer for each frame.
///
/// The caller is responsible for deallocating the command buffers when done
/// using them.
fn create_command_buffers(
    device: &Device,
    swapchain: &Swapchain,
    command_pool: &vk::CommandPool,
) -> Result<Vec<vk::CommandBuffer>> {
    let create_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(swapchain.framebuffers.len() as u32);
    let command_buffers = unsafe {
        device
            .logical_device
            .allocate_command_buffers(&create_info)?
    };

    Ok(command_buffers)
}

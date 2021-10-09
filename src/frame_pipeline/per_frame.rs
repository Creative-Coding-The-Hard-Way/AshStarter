use super::PerFrame;

use crate::vulkan::{RenderDevice, SemaphorePool};

use ::{
    anyhow::{Context, Result},
    ash::{version::DeviceV1_0, vk},
};

impl PerFrame {
    /// Create new per-frame resources.
    pub fn new(
        vk_dev: &RenderDevice,
        semaphore_pool: &mut SemaphorePool,
        frame_index: usize,
    ) -> Result<Self> {
        let acquire_semaphore = vk::Semaphore::null();
        let release_semaphore = semaphore_pool.get_semaphore(vk_dev)?;
        let queue_submit_fence = {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_fence(&create_info, None)
                    .context(format!(
                        "Unable to create fence for frame {}",
                        frame_index
                    ))?
            }
        };
        vk_dev.name_vulkan_object(
            format!("Frame {} - Queue Submit Fence", frame_index),
            vk::ObjectType::FENCE,
            queue_submit_fence,
        )?;

        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo {
                queue_family_index: vk_dev.graphics_queue.family_id,
                flags: vk::CommandPoolCreateFlags::TRANSIENT,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_command_pool(&create_info, None)
                    .context(format!(
                        "unable to create command pool for frame {}",
                        frame_index
                    ))?
            }
        };
        vk_dev.name_vulkan_object(
            format!("Frame {} - Command Pool", frame_index),
            vk::ObjectType::COMMAND_POOL,
            command_pool,
        )?;

        let command_buffer = {
            let create_info = vk::CommandBufferAllocateInfo {
                command_pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: 1,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .allocate_command_buffers(&create_info)?[0]
            }
        };
        vk_dev.name_vulkan_object(
            format!("Frame {} - Command Buffer", frame_index),
            vk::ObjectType::COMMAND_BUFFER,
            command_buffer,
        )?;

        Ok(Self {
            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            command_pool,
            command_buffer,
        })
    }

    /// Destroy per-frame resources.
    pub unsafe fn destroy(self, vk_dev: &RenderDevice) {
        if self.acquire_semaphore != vk::Semaphore::null() {
            vk_dev
                .logical_device
                .destroy_semaphore(self.acquire_semaphore, None);
        }
        if self.release_semaphore != vk::Semaphore::null() {
            vk_dev
                .logical_device
                .destroy_semaphore(self.release_semaphore, None);
        }
        vk_dev
            .logical_device
            .destroy_fence(self.queue_submit_fence, None);
        vk_dev
            .logical_device
            .free_command_buffers(self.command_pool, &[self.command_buffer]);
        vk_dev
            .logical_device
            .destroy_command_pool(self.command_pool, None);
    }
}

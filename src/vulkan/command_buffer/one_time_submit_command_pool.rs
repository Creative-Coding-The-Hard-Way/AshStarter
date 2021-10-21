use super::{
    CommandBuffer, CommandBufferError, CommandPool, OneTimeSubmitCommandPool,
};

use crate::vulkan::{
    errors::VulkanDebugError, GpuQueue, RenderDevice, VulkanDebug,
};

use ::{
    anyhow::Result,
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl OneTimeSubmitCommandPool {
    /// Create a new pool for submitting commands to the provided GPU queue.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        queue: &GpuQueue,
    ) -> Result<Self, CommandBufferError> {
        let pool = Arc::new(CommandPool::new(
            vk_dev.clone(),
            queue,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?);
        let cmd = CommandBuffer::new_primary(pool.clone())?;
        Ok(Self {
            pool,
            cmd,
            queue: *queue,
            vk_dev,
        })
    }

    /// Submit commands to the configured GPU queue. This function blocks until
    /// all commands complete.
    pub fn submit_sync_commands<Func, T>(&self, func: Func) -> Result<T>
    where
        Func: FnOnce(&Arc<RenderDevice>, vk::CommandBuffer) -> T,
    {
        self.pool.reset()?;
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            self.vk_dev
                .logical_device
                .begin_command_buffer(self.cmd.raw, &begin_info)?;

            let result: T = func(&self.vk_dev, self.cmd.raw);

            self.vk_dev
                .logical_device
                .end_command_buffer(self.cmd.raw)?;

            let submit_info = vk::SubmitInfo {
                command_buffer_count: 1,
                p_command_buffers: &self.cmd.raw,
                ..Default::default()
            };
            self.vk_dev.logical_device.queue_submit(
                self.queue.queue,
                &[submit_info],
                vk::Fence::null(),
            )?;
            self.vk_dev.logical_device.device_wait_idle()?;

            Ok(result)
        }
    }
}

impl VulkanDebug for OneTimeSubmitCommandPool {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        let name = debug_name.into();
        self.pool
            .set_debug_name(format!("{} - CommandPool", name))?;
        self.cmd
            .set_debug_name(format!("{} - CommandBuffer", name))?;
        Ok(())
    }
}

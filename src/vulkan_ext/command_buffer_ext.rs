use ::ash::{version::DeviceV1_0, vk};

use crate::{
    vulkan::CommandBuffer,
    vulkan_ext::{CommandBufferExtError, CommandResult},
};

/// Command buffer convenience methods.
pub trait CommandBufferExt {
    /// Begin recording commands into the command buffer with the
    /// `ONE_TIME_SUBMIT` flag set.
    unsafe fn begin_one_time_submit(&self) -> CommandResult<&Self>;

    /// Finish recording commands into this command buffer.
    unsafe fn end_commands(&self) -> CommandResult<()>;

    /// Finish the current renderpass.
    unsafe fn end_renderpass(&self) -> &Self;
}

impl CommandBufferExt for CommandBuffer {
    unsafe fn begin_one_time_submit(&self) -> CommandResult<&Self> {
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        self.vk_dev
            .logical_device
            .begin_command_buffer(self.raw, &begin_info)
            .map_err(CommandBufferExtError::UnableToBeginCommandBuffer)?;
        Ok(&self)
    }

    unsafe fn end_commands(&self) -> CommandResult<()> {
        self.vk_dev
            .logical_device
            .end_command_buffer(self.raw)
            .map_err(CommandBufferExtError::UnableToEndCommandBuffer)?;
        Ok(())
    }

    unsafe fn end_renderpass(&self) -> &Self {
        self.vk_dev.logical_device.cmd_end_render_pass(self.raw);
        &self
    }
}

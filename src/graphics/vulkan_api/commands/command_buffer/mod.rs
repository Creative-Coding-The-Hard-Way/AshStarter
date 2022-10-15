mod api;

use {
    crate::graphics::vulkan_api::{
        CommandPool, Fence, RenderDevice, Semaphore, VulkanDebug, VulkanError,
    },
    ash::vk,
    std::sync::Arc,
};

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
    command_pool: Arc<CommandPool>,
    render_device: Arc<RenderDevice>,
}

impl CommandBuffer {
    /// Create a new command buffer.
    pub fn new(
        render_device: Arc<RenderDevice>,
        command_pool: Arc<CommandPool>,
        command_buffer_level: vk::CommandBufferLevel,
    ) -> Result<Self, VulkanError> {
        // Safe because the buffer owns a reference to the parent command pool.
        let command_buffer = unsafe {
            command_pool.allocate_command_buffer(command_buffer_level)?
        };
        Ok(Self {
            command_buffer,
            command_pool,
            render_device,
        })
    }

    /// Get the underlying command buffer handle.
    ///
    /// # Safety
    ///
    /// Unsafe because ownership is not transferred. The caller is
    /// responsible ensuring any usage ends before this object is
    /// dropped.
    pub unsafe fn raw(&self) -> &vk::CommandBuffer {
        &self.command_buffer
    }

    /// - wait_semaphores is a set of semaphores to wait on before executing
    ///   commands.
    /// - wait_stage is the graphics pipeline stage to perform the semaphore
    ///   wait.
    /// - signal_semaphores the set of semaphores to signal when the command
    ///   buffer has finished executing
    /// - signal_fence an optional fence to signal when the command buffer has
    ///   finished executing
    ///
    /// # Safety
    ///
    /// Unsafe because the caller is responsible for ensuring that the
    /// command buffer and all referenced resources are not dropped
    /// until all submitted commands have finished executing.
    pub unsafe fn submit_graphics_commands(
        &self,
        wait_semaphores: &[&Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> Result<(), VulkanError> {
        let raw_wait_semaphores: Vec<vk::Semaphore> = wait_semaphores
            .iter()
            .map(|semaphore| *semaphore.raw())
            .collect();
        let raw_signal_semaphores: Vec<vk::Semaphore> = signal_semaphores
            .iter()
            .map(|semaphore| *semaphore.raw())
            .collect();
        let raw_signal_fence = signal_fence
            .map(|fence| *fence.raw())
            .unwrap_or(vk::Fence::null());
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffer,
            wait_semaphore_count: raw_wait_semaphores.len() as u32,
            p_wait_semaphores: if !raw_wait_semaphores.is_empty() {
                raw_wait_semaphores.as_ptr()
            } else {
                std::ptr::null()
            },
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            signal_semaphore_count: raw_signal_semaphores.len() as u32,
            p_signal_semaphores: if !raw_signal_semaphores.is_empty() {
                raw_signal_semaphores.as_ptr()
            } else {
                std::ptr::null()
            },
            ..Default::default()
        };
        self.render_device
            .submit_graphics_commands(submit_info, &raw_signal_fence)
    }

    /// - wait_semaphores is a set of semaphores to wait on before executing
    ///   commands.
    /// - wait_stage is the graphics pipeline stage to perform the semaphore
    ///   wait.
    /// - signal_semaphores the set of semaphores to signal when the command
    ///   buffer has finished executing
    /// - signal_fence an optional fence to signal when the command buffer has
    ///   finished executing
    ///
    /// # Safety
    ///
    /// Unsafe because the caller is responsible for ensuring that the
    /// command buffer and all referenced resources are not dropped
    /// until all submitted commands have finished executing.
    pub unsafe fn submit_compute_commands(
        &self,
        wait_semaphores: &[&Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> Result<(), VulkanError> {
        let raw_wait_semaphores: Vec<vk::Semaphore> = wait_semaphores
            .iter()
            .map(|semaphore| *semaphore.raw())
            .collect();
        let raw_signal_semaphores: Vec<vk::Semaphore> = signal_semaphores
            .iter()
            .map(|semaphore| *semaphore.raw())
            .collect();
        let raw_signal_fence = signal_fence
            .map(|fence| *fence.raw())
            .unwrap_or(vk::Fence::null());
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffer,
            wait_semaphore_count: raw_wait_semaphores.len() as u32,
            p_wait_semaphores: if !raw_wait_semaphores.is_empty() {
                raw_wait_semaphores.as_ptr()
            } else {
                std::ptr::null()
            },
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            signal_semaphore_count: raw_signal_semaphores.len() as u32,
            p_signal_semaphores: if !raw_signal_semaphores.is_empty() {
                raw_signal_semaphores.as_ptr()
            } else {
                std::ptr::null()
            },
            ..Default::default()
        };
        self.render_device
            .submit_compute_commands(submit_info, &raw_signal_fence)
    }
}

impl VulkanDebug for CommandBuffer {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::COMMAND_BUFFER,
            self.command_buffer,
        )
    }
}

impl Drop for CommandBuffer {
    /// # Safety
    ///
    /// The application must ensure no GPU operations still reference the
    /// command buffer when it is dropped.
    fn drop(&mut self) {
        unsafe { self.command_pool.free_command_buffer(self.command_buffer) }
    }
}

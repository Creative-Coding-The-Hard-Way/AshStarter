use std::ffi::c_void;

use ash::vk;

use super::{Allocation, RenderDevice};
use crate::graphics::vulkan_api::VulkanError;

impl RenderDevice {
    /// Stall the thread until the GPU is done with all operations.
    pub fn wait_idle(&self) -> Result<(), VulkanError> {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .map_err(VulkanError::UnableToWaitForDeviceToIdle)
        }
    }

    /// Create a raw Vulkan ImageView instance.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the ImageView is destroyed before
    /// the RenderDevice is dropped.
    pub unsafe fn create_image_view(
        &self,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, VulkanError> {
        self.logical_device
            .create_image_view(create_info, None)
            .map_err(VulkanError::UnableToCreateImageView)
    }

    /// Destroy a raw Vulkan ImageView.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the ImageView is no longer being
    /// used by any GPU operations at the time of destruction.
    pub unsafe fn destroy_image_view(&self, image_view: vk::ImageView) {
        self.logical_device.destroy_image_view(image_view, None)
    }

    /// Create a raw Vulkan Fence.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the Fence is destroyed before the
    /// RenderDevice is dropped.
    pub unsafe fn create_fence(
        &self,
        create_info: &vk::FenceCreateInfo,
    ) -> Result<vk::Fence, VulkanError> {
        self.logical_device
            .create_fence(create_info, None)
            .map_err(VulkanError::UnableToCreateFence)
    }

    /// Destroy the raw Vulkan Fence.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the Fence is no longer being used
    /// by any GPU operations at the time of destruction.
    pub unsafe fn destroy_fence(&self, fence: vk::Fence) {
        self.logical_device.destroy_fence(fence, None)
    }

    /// Wait for fences to be signaled. If wait_all is false then only one of
    /// the fences needs to be signaled. If wait_all is true then all fences
    /// must be signaled for this method to unblock.
    pub fn wait_for_fences(
        &self,
        fences: &[vk::Fence],
        wait_all: bool,
    ) -> Result<(), VulkanError> {
        unsafe {
            self.logical_device
                .wait_for_fences(fences, wait_all, u64::MAX)
                .map_err(VulkanError::UnexpectedFenceWaitError)
        }
    }

    /// Reset every fence. No-op for fences that are already in the unsignaled
    /// state.
    pub fn reset_fences(
        &self,
        fences: &[vk::Fence],
    ) -> Result<(), VulkanError> {
        unsafe {
            self.logical_device
                .reset_fences(fences)
                .map_err(VulkanError::UnexpectedFenceResetError)
        }
    }

    /// Create a Vulkan semahpore.
    ///
    /// # Safety
    ///
    /// The caller is responsible for destroying the Semaphore before the
    /// RenderDevice is dropped.
    pub unsafe fn create_semaphore(
        &self,
        create_info: &vk::SemaphoreCreateInfo,
    ) -> Result<vk::Semaphore, VulkanError> {
        self.logical_device
            .create_semaphore(create_info, None)
            .map_err(VulkanError::UnableToCreateSemaphore)
    }

    /// Destroy a vulkan semaphore.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the Semaphore is not being
    /// used by the GPU when this method is called.
    pub unsafe fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.logical_device.destroy_semaphore(semaphore, None)
    }

    /// # Safety
    ///
    /// The caller is responsible for destroying the render pass before the
    /// render device is dropped.
    pub unsafe fn create_render_pass(
        &self,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, VulkanError> {
        self.logical_device
            .create_render_pass(create_info, None)
            .map_err(VulkanError::UnableToCreateRenderPass)
    }

    /// # Safety
    ///
    /// The caller is responsible for making sure that the render pass is not
    /// in use by the GPU when it is destroyed.
    pub unsafe fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        self.logical_device.destroy_render_pass(render_pass, None)
    }

    /// # Safety
    ///
    /// The caller is responsible for destroying the framebuffer before
    /// it is dropped.
    pub unsafe fn create_framebuffer(
        &self,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, VulkanError> {
        self.logical_device
            .create_framebuffer(create_info, None)
            .map_err(VulkanError::UnableToCreateFramebuffer)
    }

    /// # Safety
    ///
    /// The caller is responsible for making sure that the framebuffer is not
    /// in use by the GPU when it is destroyed.
    pub unsafe fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        self.logical_device.destroy_framebuffer(framebuffer, None)
    }

    /// # Safety
    ///
    /// The caller is responsible for destroying the command pool before the
    /// render device is dropped.
    pub unsafe fn create_command_pool(
        &self,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<vk::CommandPool, VulkanError> {
        self.logical_device
            .create_command_pool(create_info, None)
            .map_err(VulkanError::UnableToCreateCommandPool)
    }

    /// # Safety
    ///
    /// The caller is responsible for ensuring the command pool is not in use by
    /// the GPU.
    pub unsafe fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        self.logical_device.destroy_command_pool(command_pool, None)
    }

    /// # Safety
    ///
    /// The caller is responsible for destroying the command buffers before
    /// the Render Device is dropped.
    pub unsafe fn allocate_command_buffers(
        &self,
        allocate_info: &vk::CommandBufferAllocateInfo,
    ) -> Result<Vec<vk::CommandBuffer>, VulkanError> {
        self.logical_device
            .allocate_command_buffers(allocate_info)
            .map_err(VulkanError::UnableToAllocateCommandBuffers)
    }

    /// # Safety
    ///
    /// Unsafe because the caller must ensure none of the command buffers is
    /// being used by the GPU when freed.
    pub unsafe fn free_command_buffers(
        &self,
        command_pool: &vk::CommandPool,
        command_buffers: &[vk::CommandBuffer],
    ) {
        self.logical_device
            .free_command_buffers(*command_pool, command_buffers)
    }

    /// # Safety
    ///
    /// Unsafe because the caller must ensure none of the allocated command
    /// buffers is being used by the GPU when the pool is reset.
    pub unsafe fn reset_command_pool(
        &self,
        command_pool: &vk::CommandPool,
        flags: vk::CommandPoolResetFlags,
    ) -> Result<(), VulkanError> {
        self.logical_device
            .reset_command_pool(*command_pool, flags)
            .map_err(VulkanError::UnableToResetCommandPool)
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), VulkanError> {
        unsafe {
            self.logical_device
                .begin_command_buffer(*command_buffer, begin_info)
                .map_err(VulkanError::UnableToBeginCommandBuffer)
        }
    }

    pub fn end_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
    ) -> Result<(), VulkanError> {
        unsafe {
            self.logical_device
                .end_command_buffer(*command_buffer)
                .map_err(VulkanError::UnableToEndCommandBuffer)
        }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: &vk::CommandBuffer) {
        unsafe {
            self.logical_device.cmd_end_render_pass(*command_buffer);
        }
    }

    /// # Safety
    ///
    /// Unsafe because the caller must ensure that the relevant render pass
    /// lives at least until this command has finished executing on the GPU.
    pub unsafe fn cmd_begin_render_pass(
        &self,
        command_buffer: &vk::CommandBuffer,
        render_pass_begin_info: &vk::RenderPassBeginInfo,
        subpass_contents: vk::SubpassContents,
    ) {
        self.logical_device.cmd_begin_render_pass(
            *command_buffer,
            render_pass_begin_info,
            subpass_contents,
        )
    }

    /// - signal_fence is an optional handle to a fence which will be
    ///   signaled once all submitted command buffers have finished
    ///   execution.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure that the command buffer
    /// being submitted and all associated resources live until the
    /// graphics commands finish executing.
    pub unsafe fn submit_graphics_commands(
        &self,
        submit_info: vk::SubmitInfo,
        signal_fence: &vk::Fence,
    ) -> Result<(), VulkanError> {
        self.logical_device
            .queue_submit(
                self.graphics_queue.raw_queue(),
                &[submit_info],
                *signal_fence,
            )
            .map_err(VulkanError::UnableToSubmitGraphicsCommands)
    }

    /// Map a piece of device memory to a host-accessible pointer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - only memmory accessible by the host can be mapped
    ///  - memory that is not HOST_COHERENT requires additional synchronization
    ///    after writes/reads
    ///  - the application is responsible for making a corresponding call to
    ///    unmap
    ///  - device memory can only be mapped ONCE even if the offset and size
    ///    would result in disjoint regions being mapped
    pub unsafe fn map_memory(
        &self,
        device_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> Result<*mut c_void, VulkanError> {
        self.logical_device
            .map_memory(
                device_memory,
                offset,
                size,
                vk::MemoryMapFlags::empty(),
            )
            .map_err(VulkanError::UnableToMapDeviceMemory)
    }

    /// Unmap a piece of device memory.
    ///
    /// # Safety
    ///
    /// Unsafe because the application must ensure the mapped pointer is not
    /// still being used.
    pub unsafe fn unmap_memory(&self, device_memory: vk::DeviceMemory) {
        self.logical_device.unmap_memory(device_memory);
    }

    /// Create a new Vulkan buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must destroy the buffer before the render device is
    ///    dropped
    pub unsafe fn create_buffer(
        &self,
        create_info: &vk::BufferCreateInfo,
    ) -> Result<vk::Buffer, VulkanError> {
        self.logical_device
            .create_buffer(create_info, None)
            .map_err(|err| {
                VulkanError::UnableToCreateBuffer(
                    create_info.size,
                    create_info.usage,
                    err,
                )
            })
    }

    /// Destroy a Vulkan buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must ensure no Device operations still depend on the
    ///    buffer
    pub unsafe fn destroy_buffer(&self, buffer: vk::Buffer) {
        self.logical_device.destroy_buffer(buffer, None)
    }

    /// Get the memory allocation requirements for the buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must ensure that the buffer has not previously been
    ///    freed back to the device.
    pub unsafe fn get_buffer_memory_requirements(
        &self,
        buffer: &vk::Buffer,
    ) -> vk::MemoryRequirements {
        self.logical_device.get_buffer_memory_requirements(*buffer)
    }

    /// Bind an allocation to a buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must ensure the allocation and buffer have the same
    ///    lifetime
    pub unsafe fn bind_buffer_memory(
        &self,
        buffer: &vk::Buffer,
        allocation: &Allocation,
    ) -> Result<(), VulkanError> {
        self.logical_device
            .bind_buffer_memory(
                *buffer,
                allocation.device_memory(),
                allocation.offset_in_bytes(),
            )
            .map_err(VulkanError::UnableToBindBufferMemory)
    }

    /// Flush mapped memory so writes on the host are visible on the device.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must ensure the mapped ranges are correct
    pub unsafe fn flush_mapped_memory_ranges(
        &self,
        ranges: &[vk::MappedMemoryRange],
    ) -> Result<(), VulkanError> {
        self.logical_device
            .flush_mapped_memory_ranges(ranges)
            .map_err(VulkanError::UnableToFlushMappedMemoryRanges)
    }

    /// Create a new Vulkan shader module.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must destroy the shader module before the render device
    ///    is dropped.
    pub unsafe fn create_shader_module(
        &self,
        create_info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, VulkanError> {
        self.logical_device
            .create_shader_module(create_info, None)
            .map_err(VulkanError::UnableToCreateShaderModule)
    }

    /// Destroy a Vulkan shader module.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller must ensure no GPU operations still reference the shader
    ///    when it is destroyed
    pub unsafe fn destroy_shader_module(
        &self,
        shader_module: vk::ShaderModule,
    ) {
        self.logical_device
            .destroy_shader_module(shader_module, None)
    }

    /// Create a new graphics pipeline.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the application is responsible for destroying the pipeline before
    ///    the render device is dropped.
    pub unsafe fn create_graphics_pipeline(
        &self,
        create_info: &vk::GraphicsPipelineCreateInfo,
    ) -> Result<vk::Pipeline, VulkanError> {
        let pipeline_cache = vk::PipelineCache::default();
        let pipelines_result = self.logical_device.create_graphics_pipelines(
            pipeline_cache,
            &[*create_info],
            None,
        );
        match pipelines_result {
            Ok(pipelines) => Ok(pipelines[0]),
            Err((_, result)) => {
                Err(VulkanError::UnableToCreateGraphicsPipeline(result))
            }
        }
    }

    /// Destroy a pipeline.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the caller must ensure that the pipeline is not being used by
    ///     the GPU at the time of destruction.
    pub unsafe fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        self.logical_device.destroy_pipeline(pipeline, None);
    }

    /// Create a pipeline layout.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The caller is responsible for destroying the pipeline layout
    ///     before the Render Device is destroyed.
    pub unsafe fn create_pipeline_layout(
        &self,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<vk::PipelineLayout, VulkanError> {
        self.logical_device
            .create_pipeline_layout(create_info, None)
            .map_err(VulkanError::UnableToCreatePipelineLayout)
    }

    /// Destroy a pipeline layout.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The caller must ensure the GPU is not using the pipeline layout at
    ///     the time of destruction.
    pub unsafe fn destroy_pipeline_layout(
        &self,
        pipeline_layout: vk::PipelineLayout,
    ) {
        self.logical_device
            .destroy_pipeline_layout(pipeline_layout, None);
    }

    /// Bind a pipeline for rendering.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not destroy the pipeline while any command
    ///     buffers still reference it.
    pub unsafe fn cmd_bind_pipeline(
        &self,
        command_buffer: &vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline: &vk::Pipeline,
    ) {
        self.logical_device.cmd_bind_pipeline(
            *command_buffer,
            pipeline_bind_point,
            *pipeline,
        );
    }

    /// Set a viewport for rendering commands.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - first viewport must be between 0 and the number of viewports
    ///     provided
    ///   - first viewport must be 0 if multiviewports are not enabled
    pub unsafe fn cmd_set_viewport(
        &self,
        command_buffer: &vk::CommandBuffer,
        first_viewport: u32,
        viewports: &[vk::Viewport],
    ) {
        self.logical_device.cmd_set_viewport(
            *command_buffer,
            first_viewport,
            viewports,
        );
    }

    /// Set scissor rectangles for rendering.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - first scissor must be between 0 and the number of scissors provided
    ///   - only one scissor can be specified if multiViewport feature is not
    ///     enabled
    pub unsafe fn cmd_set_scissor(
        &self,
        command_buffer: &vk::CommandBuffer,
        first_scissor: u32,
        scissors: &[vk::Rect2D],
    ) {
        self.logical_device.cmd_set_scissor(
            *command_buffer,
            first_scissor,
            scissors,
        )
    }

    /// Draw vertices.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - there must be a pipeline bound
    ///   - all of the relevant buffers and settings must be set in the command
    ///     buffer prior to issuing a draw command
    pub unsafe fn cmd_draw(
        &self,
        command_buffer: &vk::CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        self.logical_device.cmd_draw(
            *command_buffer,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        );
    }

    /// Bind vertex buffers for drawing.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - first_binding must be in the range 0..buffers.len()
    ///   - there must be one offset for each buffer in buffers
    ///   - each offset must be between 0 and the length of the respective
    ///     buffer
    pub unsafe fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: &vk::CommandBuffer,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[u64],
    ) {
        self.logical_device.cmd_bind_vertex_buffers(
            *command_buffer,
            first_binding,
            buffers,
            offsets,
        )
    }
}

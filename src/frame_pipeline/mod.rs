mod frame_pipeline;
mod per_frame;

use crate::vulkan::{
    errors::VulkanError,
    sync::{Fence, Semaphore, SemaphorePool},
    CommandBuffer, CommandPool, RenderDevice,
};

use ::{std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),

    #[error(transparent)]
    UnexpectedVulkanError(#[from] VulkanError),
}

/// A frame pipeline aids with the swapchain acquire->render->present workflow.
pub struct FramePipeline {
    frames: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,

    /// The device used to create this frame pipeline.
    pub vk_dev: Arc<RenderDevice>,
}

/// All per-frame resources required for coordinating the swapchain with
/// multiple frames in-flight.
pub struct PerFrame {
    /// Signalled when the frame is ready to be used for rendering.
    pub acquire_semaphore: Option<Semaphore>,

    /// Signalled when all graphics operations are complete and the frame is
    /// ready for presentation.
    pub release_semaphore: Semaphore,

    /// Signalled when all submitted graphics commands have completed.
    pub queue_submit_fence: Fence,

    /// The command buffer for operations in this frame.
    pub command_buffer: CommandBuffer,

    /// The command pool for operations in this frame.
    pub command_pool: Arc<CommandPool>,
}

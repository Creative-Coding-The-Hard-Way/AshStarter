mod frame_pipeline;
mod per_frame;

use crate::vulkan::SemaphorePool;

use ::{ash::vk, thiserror::Error};

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),
}

/// A frame pipeline aids with the swapchain acquire->render->present workflow.
pub struct FramePipeline {
    frames: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,
}

/// All per-frame resources required for coordinating the swapchain with
/// multiple frames in-flight.
pub struct PerFrame {
    /// Signalled when the frame is ready to be used for rendering.
    pub acquire_semaphore: vk::Semaphore,

    /// Signalled when all graphics operations are complete and the frame is
    /// ready for presentation.
    pub release_semaphore: vk::Semaphore,

    /// Signalled when all submitted graphics commands have completed.
    pub queue_submit_fence: vk::Fence,

    /// The command pool for operations in this frame.
    pub command_pool: vk::CommandPool,

    /// The command buffer for operations in this frame.
    pub command_buffer: vk::CommandBuffer,
}

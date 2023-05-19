use {super::FrameSync, ash::vk};

/// The current animation Frame.
///
/// The Frame does not own any resources and it is an error to retain copies of
/// any of the Frame's resource handle after calling `present_frame`.
#[derive(Debug)]
pub struct Frame {
    sync: FrameSync,
    swapchain_image_index: usize,
}

// Public API
// ----------

impl Frame {
    /// The primary command buffer for the current frame.
    ///
    /// The buffer is already started when given to a Frame so commands can
    /// be freely added without any additional setup.
    ///
    /// The buffer is submitted automatically when the frame is returned for
    /// presentation.
    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.sync.command_pool.primary_command_buffer(0)
    }

    /// The current frame's index. Always in the range [0-N) where N is the
    /// number of frames in flight.
    pub fn frame_index(&self) -> usize {
        self.sync.index
    }

    /// The index of the swapchain image being targeted by this frame.
    pub fn swapchain_image_index(&self) -> usize {
        self.swapchain_image_index
    }
}

// Private API
// -----------

impl Frame {
    /// Create a new Frame which the Application can use to schedule graphics
    /// commands.
    pub(super) fn new(sync: FrameSync, swapchain_image_index: usize) -> Self {
        Self {
            sync,
            swapchain_image_index,
        }
    }

    pub(super) fn take_sync(self) -> FrameSync {
        self.sync
    }
}

//! Just the logic for acquiring and presenting swapchain images.
//!
//! It's nice to bundle this logic up into one spot - even if it is all still
//! exposed on the public api - because it ends up being so verbose.

use {
    super::Swapchain, crate::graphics::GraphicsError, anyhow::Context, ash::vk,
    ccthw_ash_instance::VulkanHandle,
};

/// Indicates that the swapchain needs a rebuild, or that the image was acquired
/// successfully.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SwapchainStatus {
    /// Completed the operation with the given swapchain index.
    Index(usize),

    /// Indicates that the swapchain needs to be rebuilt.
    NeedsRebuild,
}

// Public API
// ----------

impl Swapchain {
    /// Acquire the next swapchain image.
    ///
    /// # Params
    ///
    /// * `semaphore` - a semaphore to signal when the swapchain image is
    ///   available.
    ///
    /// # Safety
    ///
    /// The application must correctly handle a swapchain acquisition failure
    /// and rebuild the swapchain on demand.
    pub unsafe fn acquire_swapchain_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<SwapchainStatus, GraphicsError> {
        let result = self.swapchain_loader.acquire_next_image(
            self.swapchain,
            std::u64::MAX,
            semaphore,
            fence,
        );
        match result {
            // index acquired and the swapchain is optimal
            Ok((index, false)) => Ok(SwapchainStatus::Index(index as usize)),

            // index acquired but the swapchain is suboptimal for the surface
            Ok((_, true)) => {
                log::debug!(
                    "Acquire Image: Swapchain suboptimal, needs rebuild."
                );
                Ok(SwapchainStatus::NeedsRebuild)
            }

            // the swapchain is lost and needs to be rebuilt
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Acquire Image: Swapchain lost, needs rebuild.");
                Ok(SwapchainStatus::NeedsRebuild)
            }

            Err(_) => Err(GraphicsError::RuntimeError(
                result
                    .context(
                        "Unexpected error while acquiring swapchain image!",
                    )
                    .err()
                    .unwrap(),
            )),
        }
    }

    /// Present a swapchain image to the screen.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must correctly handle a swapchain acquisition
    ///     failure and rebuild the swapchain on demand
    ///   - the application must transition the swapchain image to the correct
    ///     image layout. Typically this is done with a Render Pass.
    pub unsafe fn present_swapchain_image(
        &self,
        index: usize,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<SwapchainStatus, GraphicsError> {
        let index_u32 = index as u32;
        let present_info = vk::PresentInfoKHR {
            p_wait_semaphores: wait_semaphores.as_ptr(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_swapchains: &self.swapchain,
            swapchain_count: 1,
            p_image_indices: &index_u32,
            ..Default::default()
        };
        let result = self.swapchain_loader.queue_present(
            *self.render_device.presentation_queue().raw(),
            &present_info,
        );
        match result {
            // presentation succeeded and the swapchain is still optimal
            Ok(false) => Ok(SwapchainStatus::Index(index)),

            // presentation succeeded but the swapchain is submoptimal
            Ok(true) => {
                log::debug!(
                    "Present Image: Swapchain is suboptimal and needs rebuild"
                );
                Ok(SwapchainStatus::NeedsRebuild)
            }

            // the swapchain is lost and needs to be rebuilt
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Present Image: Swapchain lost, needs rebuild.");
                Ok(SwapchainStatus::NeedsRebuild)
            }

            Err(_) => Err(GraphicsError::RuntimeError(
                result
                    .context(
                        "Unexpected error while presenting swapchain image!",
                    )
                    .err()
                    .unwrap(),
            )),
        }
    }
}

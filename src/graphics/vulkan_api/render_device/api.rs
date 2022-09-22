use ash::vk;

use super::RenderDevice;
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
}

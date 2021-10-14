use super::{RenderPass, RenderPassError};

use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl RenderPass {
    /// Create a new render pass with the given create info.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<Self, RenderPassError> {
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_render_pass(create_info, None)
                .map_err(RenderPassError::UnableToCreateRenderPass)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl VulkanDebug for RenderPass {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::RENDER_PASS,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for RenderPass {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_render_pass(self.raw, None);
        }
    }
}

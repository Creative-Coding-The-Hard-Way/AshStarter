use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    anyhow::Context,
    ash::vk,
    std::sync::Arc,
};

pub struct Pipeline {
    raw: vk::Pipeline,
    render_device: Arc<RenderDevice>,
}

impl Pipeline {
    /// Create a new Vulkan resource which is automatically
    /// destroyed when dropped.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not drop the resource while it is in use by the
    ///     GPU.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        pipeline: vk::Pipeline,
    ) -> Result<Self, GraphicsError> {
        Ok(Self {
            raw: pipeline,
            render_device,
        })
    }

    /// Create a new graphics pipeline Vulkan resource which is automatically
    /// destroyed when dropped.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not drop the resource while it is in use by the
    ///     GPU.
    pub unsafe fn new_graphics_pipeline(
        render_device: Arc<RenderDevice>,
        create_info: vk::GraphicsPipelineCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let result = render_device.device().create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[create_info],
            None,
        );
        let pipeline = match result {
            Ok(mut pipelines) => pipelines.pop().unwrap(),
            Err((_, result)) => {
                return Err(GraphicsError::VulkanError(result))
                    .context("Error creating graphics pipeline")?;
            }
        };
        Self::new(render_device, pipeline)
    }

    /// Set the debug name for how this resource appears in Vulkan logs.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.raw(),
            vk::ObjectType::PIPELINE,
            name,
        )
    }

    /// Get the raw Vulkan ImageView handle.
    pub fn raw(&self) -> vk::Pipeline {
        self.raw
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.render_device.device().destroy_pipeline(self.raw, None);
        }
    }
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorSetLayout")
            .field("raw", &self.raw)
            .finish()
    }
}

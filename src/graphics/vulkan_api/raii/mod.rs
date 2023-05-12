mod buffer;
mod command_pool;
mod descriptor_pool;
mod descriptor_set_layout;
mod pipeline;
mod pipeline_layout;
mod shader_module;

use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

pub use self::{
    buffer::Buffer, command_pool::CommandPool, descriptor_pool::DescriptorPool,
    descriptor_set_layout::DescriptorSetLayout, pipeline::Pipeline,
    pipeline_layout::PipelineLayout, shader_module::ShaderModule,
};

macro_rules! raii_wrapper {
    (
        $vk_type:ident,
        $vk_create_info:ident,
        $object_type:ident,
        $create:ident,
        $destroy:ident
    ) => {
        pub struct $vk_type {
            raw: vk::$vk_type,
            render_device: Arc<RenderDevice>,
        }

        impl $vk_type {
            /// Create a new Vulkan resource which is automatically
            /// destroyed when dropped.
            ///
            /// # Safety
            ///
            /// Unsafe because:
            ///   - The application must not drop the resource while it is in
            ///     use by the GPU.
            pub unsafe fn new(
                render_device: Arc<RenderDevice>,
                create_info: &vk::$vk_create_info,
            ) -> Result<Self, GraphicsError> {
                let raw = unsafe {
                    render_device.device().$create(create_info, None)?
                };
                Ok(Self { raw, render_device })
            }

            /// Set the debug name for how this resource appears in Vulkan logs.
            pub fn set_debug_name(&self, name: impl Into<String>) {
                self.render_device.set_debug_name(
                    self.raw(),
                    vk::ObjectType::$object_type,
                    name,
                )
            }

            /// Get the raw Vulkan ImageView handle.
            pub fn raw(&self) -> vk::$vk_type {
                self.raw
            }
        }

        impl Drop for $vk_type {
            fn drop(&mut self) {
                unsafe {
                    self.render_device.device().$destroy(self.raw, None);
                }
            }
        }

        impl std::fmt::Debug for $vk_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("DescriptorSetLayout")
                    .field("raw", &self.raw)
                    .finish()
            }
        }
    };
}

pub(crate) use raii_wrapper;

raii_wrapper!(Fence, FenceCreateInfo, FENCE, create_fence, destroy_fence);
raii_wrapper!(
    Framebuffer,
    FramebufferCreateInfo,
    FRAMEBUFFER,
    create_framebuffer,
    destroy_framebuffer
);
raii_wrapper!(
    ImageView,
    ImageViewCreateInfo,
    IMAGE_VIEW,
    create_image_view,
    destroy_image_view
);
raii_wrapper!(
    RenderPass,
    RenderPassCreateInfo,
    RENDER_PASS,
    create_render_pass,
    destroy_render_pass
);
raii_wrapper!(
    Semaphore,
    SemaphoreCreateInfo,
    SEMAPHORE,
    create_semaphore,
    destroy_semaphore
);

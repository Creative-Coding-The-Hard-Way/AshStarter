mod selection;

use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{
    Fence, RenderDevice, Semaphore, VulkanError,
};

pub enum SwapchainStatus {
    ImageAcquired(usize),
    NeedsRebuild,
}

/// The swapchain and all related resources.
pub struct Swapchain {
    _images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    loader: ash::extensions::khr::Swapchain,
    swapchain_khr: vk::SwapchainKHR,
    extent: vk::Extent2D,
    render_device: Arc<RenderDevice>,
}

impl Swapchain {
    /// Create a new Swapchain, accounting for the previous swapchain if one
    /// existed.
    pub fn new(
        render_device: Arc<RenderDevice>,
        framebuffer_size: (u32, u32),
        previous: Option<Self>,
    ) -> Result<Self, VulkanError> {
        let format = selection::choose_surface_format(&render_device);
        let mode = selection::choose_present_mode(&render_device);
        let extent =
            selection::choose_swap_extent(&render_device, framebuffer_size)?;
        let image_count = selection::choose_image_count(&render_device)?;

        let mut create_info = vk::SwapchainCreateInfoKHR {
            // it is safe to use the surface KHR reference here because the
            // swapchain keeps a reference to the RenderDevice until dropped.
            surface: unsafe { render_device.surface_khr() },

            // image settings
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            min_image_count: image_count,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            // window system presentation settings
            present_mode: mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if previous.is_some() {
                previous.as_ref().unwrap().swapchain_khr
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: 1,

            ..Default::default()
        };

        let indices = render_device.swapchain_queue_family_indices();

        if indices.len() == 1 {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        }

        let loader = render_device.create_swapchain_loader();
        let swapchain_khr = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(VulkanError::UnableToCreateSwapchain)?
        };
        let images = unsafe {
            loader
                .get_swapchain_images(swapchain_khr)
                .map_err(VulkanError::UnableToGetSwapchainImages)?
        };
        for (i, image) in images.iter().enumerate() {
            render_device.name_vulkan_object(
                format!("swapchain image {}", i),
                vk::ObjectType::IMAGE,
                *image,
            );
        }
        let image_views = selection::create_image_views(
            &render_device,
            &images,
            format.format,
        )?;

        Ok(Self {
            _images: images,
            image_views,
            loader,
            swapchain_khr,
            extent,
            render_device,
        })
    }

    /// Get the 2D extent used to create the swapchain images and views.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// Get the number of swapchain images.
    pub fn swapchain_image_count(&self) -> u32 {
        self.image_views.len() as u32
    }

    /// Acquire the next swapchain image for rendering.
    /// A semaphore or fence can optionally be used to signal when the image is
    /// ready to be rendered.
    pub fn acquire_next_swapchain_image(
        &self,
        semaphore: Option<&Semaphore>,
        fence: Option<&Fence>,
    ) -> Result<SwapchainStatus, VulkanError> {
        let result = unsafe {
            let semaphore_handle = semaphore
                .map(|semaphore| *semaphore.raw())
                .unwrap_or(vk::Semaphore::null());
            let fence_handle =
                fence.map(|fence| *fence.raw()).unwrap_or(vk::Fence::null());
            self.loader.acquire_next_image(
                self.swapchain_khr,
                u64::MAX,
                semaphore_handle,
                fence_handle,
            )
        };
        match result {
            Ok((index, false)) => {
                Ok(SwapchainStatus::ImageAcquired(index as usize))
            }

            // happens when the swapchain is suboptimal for the current device
            Ok((_, true)) => Ok(SwapchainStatus::NeedsRebuild),

            // the swapchain has been lost and needs rebuilt
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                Ok(SwapchainStatus::NeedsRebuild)
            }

            Err(error) => {
                Err(VulkanError::UnableToAcquireSwapchainImage(error))
            }
        }
    }

    /// Present the swapchain image to the screen.
    pub fn present_swapchain_image(
        &self,
        index: usize,
        semaphore: &Semaphore,
    ) -> Result<(), VulkanError> {
        let index_u32 = index as u32;
        let wait_semaphores = [unsafe { *semaphore.raw() }];
        let present_info = vk::PresentInfoKHR {
            swapchain_count: 1,
            p_swapchains: &self.swapchain_khr,
            p_image_indices: &index_u32,
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            ..Default::default()
        };
        unsafe {
            self.loader
                .queue_present(
                    self.render_device.present_queue(),
                    &present_info,
                )
                .map_err(VulkanError::UnableToPresentSwapchainImage)?;
        }
        Ok(())
    }
}

impl Drop for Swapchain {
    /// # Safety
    ///
    /// The application must ensure that all usage of the Swapchain is complete
    /// before dropping.
    fn drop(&mut self) {
        unsafe {
            for &image_view in &self.image_views {
                self.render_device.destroy_image_view(image_view);
            }
            self.loader.destroy_swapchain(self.swapchain_khr, None);
        }
    }
}

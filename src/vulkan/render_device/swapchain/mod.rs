mod images;
mod selection;

use ::{
    anyhow::Result,
    ash::{version::DeviceV1_0, vk},
};

use super::{RenderDevice, Swapchain, SwapchainError};

impl RenderDevice {
    /// Perform some action with the swapchain.
    ///
    /// # Panic
    ///
    /// If the swapchain does not exist or if the swapchain mutex cannot be
    /// acquired for any reason.
    pub fn with_swapchain<ReturnType, Func>(&self, func: Func) -> ReturnType
    where
        Func: FnOnce(&Swapchain) -> ReturnType,
    {
        let swapchain = self
            .swapchain
            .lock()
            .expect("Unable to lock the swapchain mutex");
        let borrow = swapchain.as_ref().expect("The swapchain does not exist");
        func(borrow)
    }

    /// Returns the number of images in the swapchain.
    pub fn swapchain_image_count(&self) -> u32 {
        self.with_swapchain(|swapchain| swapchain.image_views.len() as u32)
    }

    /// Acquire the next swapchain image index.
    ///
    /// # Params
    ///
    /// * `semaphore` - the semaphore to signal when the swapchain image is
    ///    available for rendering. Can be null if uneeded.
    /// * `fence` - the fence to signal when the swapchain image is available
    ///    for rendering. Can be null if uneeded.
    pub fn acquire_next_swapchain_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<usize, SwapchainError> {
        self.with_swapchain(|swapchain| {
            let result = unsafe {
                swapchain.loader.acquire_next_image(
                    swapchain.khr,
                    u64::MAX,
                    semaphore,
                    fence,
                )
            };
            if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
                return Err(SwapchainError::NeedsRebuild);
            }
            if let Ok((_, true)) = result {
                return Err(SwapchainError::NeedsRebuild);
            }
            let (index, _) = result.ok().unwrap();
            Ok(index as usize)
        })
    }

    /// Rebuild the render device's swapchain with the provided framebuffer
    /// size. Automatically handles replacing an existing swapchain if one
    /// already exists.
    ///
    /// # WARNING:
    ///
    /// There is no internal synchronization. The application
    /// must ensure that there are no in-progress operations using the
    /// swapchain when it is replaced. This operation will block on the graphics
    /// and presentation queues draining completely, but this could be dangerous
    /// if there are out-of-order semaphore waits still pending on either queue.
    ///
    /// e.g. if either queue contains (or *could* contain) a timeline semaphore
    /// wait, then make sure the corresponding signal is already queued, or
    /// else manually signal the semaphore to allow forward progress.
    ///
    pub fn rebuild_swapchain(
        &self,
        framebuffer_size: (u32, u32),
    ) -> Result<(), SwapchainError> {
        let mut current_swapchain = self
            .swapchain
            .lock()
            .expect("Unable to lock the swapchain mutex");

        let format = self.choose_surface_format();
        let present_mode = self.choose_present_mode();
        let extent = self.choose_swap_extent(framebuffer_size)?;
        let image_count = self.choose_image_count()?;

        let mut create_info = vk::SwapchainCreateInfoKHR {
            surface: self.window_surface.khr,

            // image settings
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            min_image_count: image_count,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            // window system presentation settings
            present_mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if current_swapchain.is_some() {
                current_swapchain.as_ref().unwrap().khr
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: 1,
            ..Default::default()
        };

        let indices =
            &[self.graphics_queue.family_id, self.present_queue.family_id];

        if self.present_queue.is_same(&self.graphics_queue) {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        };

        let loader = ash::extensions::khr::Swapchain::new(
            &self.instance.ash,
            &self.logical_device,
        );
        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(SwapchainError::UnableToCreateSwapchain)?
        };

        let swapchain_images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .map_err(SwapchainError::UnableToGetSwapchainImages)?
        };

        let image_views =
            self.create_image_views(format.format, &swapchain_images)?;

        let previous_swapchain = current_swapchain.replace(Swapchain {
            loader,
            khr: swapchain,
            image_views,
            format: format.format,
            color_space: format.color_space,
            extent,
        });

        if let Some(old_swapchain) = previous_swapchain {
            unsafe { self.destroy_swapchain(old_swapchain)? };
        }

        Ok(())
    }

    /// UNSAFE: because there is no internal synchronization. The application
    /// must ensure that there are no in-progress operations using the
    /// swapchain when it is dropped. Drop will block on both queues draining,
    /// but this could be dangerous if there are out-of-order semaphore waits
    /// still pending.
    pub(super) unsafe fn destroy_swapchain(
        &self,
        swapchain: Swapchain,
    ) -> Result<(), SwapchainError> {
        self.logical_device
            .queue_wait_idle(self.graphics_queue.queue)
            .map_err(SwapchainError::UnableToDrainGraphicsQueue)?;
        self.logical_device
            .queue_wait_idle(self.present_queue.queue)
            .map_err(SwapchainError::UnableToDrainPresentQueue)?;
        self.logical_device
            .device_wait_idle()
            .map_err(SwapchainError::UnableToWaitForDeviceIdle)?;

        for view in swapchain.image_views {
            self.logical_device.destroy_image_view(view, None);
        }

        swapchain.loader.destroy_swapchain(swapchain.khr, None);

        Ok(())
    }
}

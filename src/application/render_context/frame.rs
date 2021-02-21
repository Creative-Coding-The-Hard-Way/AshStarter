mod sync;

use self::sync::FrameSync;
use crate::rendering::Device;

use anyhow::Result;

use std::sync::Arc;

pub struct Frame {
    pub sync: FrameSync,
    device: Arc<Device>,
}

impl Frame {
    /// Create a collection of frames with resource debug names based on the
    /// frame index.
    pub fn create_n_frames(
        device: &Arc<Device>,
        count: usize,
    ) -> Result<Vec<Self>> {
        let mut result = vec![];
        for i in 0..count {
            result.push(Self::new(device.clone(), format!("Frame {}", i))?);
        }
        Ok(result)
    }

    /// Create a new frame
    pub fn new<Name>(device: Arc<Device>, name: Name) -> Result<Self>
    where
        Name: Into<String>,
    {
        let sync = FrameSync::new(&device, name)?;
        Ok(Self { sync, device })
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { self.sync.destroy(&self.device) }
    }
}

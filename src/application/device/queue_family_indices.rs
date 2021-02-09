use anyhow::{Context, Result};

pub struct QueueFamilyIndices {
    graphics_queue: u32,
}

impl QueueFamilyIndices {
    pub fn new() -> Result<Self> {
        Ok(Self { graphics_queue: 1 })
    }
}

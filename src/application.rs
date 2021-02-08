use anyhow::Result;

pub struct Application {}

impl Application {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn run(self) -> Result<()> {
        self.init_vulkan();
        self.main_loop();
        Ok(())
    }

    fn init_vulkan(&self) {}

    fn main_loop(&self) {}
}

impl Drop for Application {
    fn drop(&mut self) {
        log::debug!("cleanup application");
    }
}

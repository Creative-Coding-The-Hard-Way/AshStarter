use anyhow::Result;
use ccthw::{application::Application, logging};

fn main() -> Result<()> {
    logging::setup()?;
    Application::new().run()
}

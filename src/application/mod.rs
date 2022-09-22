use anyhow::Result;

/// The main application state.
pub struct Application {
    running: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    /// Create a new running application.
    pub fn new() -> Self {
        Self { running: true }
    }

    /// Run the application until exit.
    pub fn run(mut self) -> Result<()> {
        while self.running {
            self.update()?;
        }

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        use std::io::{self, Write};

        println!("Should the app continue to run? (y\\n)");
        print!("-> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        line = line.trim().to_string();

        println!("got {}", line);
        println!("-----------------");

        if line == "n" {
            self.running = false;
        }

        Ok(())
    }
}

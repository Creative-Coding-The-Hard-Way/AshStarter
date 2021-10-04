//! This module defines the main application initialization, event loop, and
//! rendering.
//!
use anyhow::Result;

// The main application state.
pub struct Application {
    line: String,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        Ok(Self {
            line: String::from("y"),
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        while self.line.eq_ignore_ascii_case("y") {
            self.update()?;
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        use std::io::{self, Write};

        // write the prompt
        println!("Should the app continue to run? (y\\n)");
        print!("-> ");
        let _ = io::stdout().flush();

        // get input
        self.line.clear();
        io::stdin().read_line(&mut self.line)?;
        self.line = self.line.trim().to_string();

        // update!
        println!("you said '{}'", self.line);
        println!("----------------------------------------------------------");
        println!("");
        Ok(())
    }
}

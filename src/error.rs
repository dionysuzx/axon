use std::io::{self, Write};

#[derive(Debug)]
pub struct CliError {
    pub code: i32,
    pub message: String,
}

impl CliError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn print(&self) {
        if self.message.is_empty() {
            return;
        }
        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "{}", self.message);
    }
}

//! Common test utilities and shared infrastructure
//! 
//! This module contains shared code used across multiple test modules.
//! Some items may appear unused in individual modules but are used by others.

#![allow(unused_imports)] // Shared module - not all imports used in each test file
#![allow(dead_code)]     // Shared utilities - not all functions used in each test file

use std::io::{self, Read, Write};
use std::process::Command;
use std::time::{Duration, Instant};

pub use workspace::*;
pub use verification::*;
pub use generators::*;

mod workspace;
mod verification;
mod generators;

// Helper function for real-time logging
pub fn log(msg: &str) {
    // Using stderr because stdout is buffered by test framework even with --nocapture
    eprintln!("{}", msg);
    io::stderr().flush().unwrap();
}

// Helper function to run command with timeout (no threading)
pub fn run_command_with_timeout(
    mut cmd: Command,
    timeout_secs: u64,
) -> anyhow::Result<std::process::Output> {
    use std::process::Stdio;

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    // Poll for completion with timeout
    loop {
        match child.try_wait()? {
            Some(status) => {
                let stdout = {
                    let mut buf = Vec::new();
                    if let Some(ref mut stdout) = child.stdout {
                        stdout.read_to_end(&mut buf)?;
                    }
                    buf
                };
                let stderr = {
                    let mut buf = Vec::new();
                    if let Some(ref mut stderr) = child.stderr {
                        stderr.read_to_end(&mut buf)?;
                    }
                    buf
                };
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            None => {
                if start.elapsed() > timeout {
                    // Kill the process
                    let _ = child.kill();
                    anyhow::bail!("Command timed out after {:?}", timeout);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

// Project types that can be added
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Astro,
    React,
    DurableObject,
    Crate,
}

impl ProjectType {
    #[allow(dead_code)] // Used by different test modules
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Astro => "astro",
            ProjectType::React => "react",
            ProjectType::DurableObject => "durable-object",
            ProjectType::Crate => "crate",
        }
    }

    #[allow(dead_code)] // Used by different test modules
    pub fn is_typescript(&self) -> bool {
        matches!(
            self,
            ProjectType::Astro | ProjectType::React | ProjectType::DurableObject
        )
    }

    #[allow(dead_code)] // Used by different test modules
    pub fn is_crate(&self) -> bool {
        matches!(self, ProjectType::Crate)
    }

    #[allow(dead_code)] // Used by different test modules
    pub fn directory(&self) -> &'static str {
        match self {
            ProjectType::Astro => "sites",
            ProjectType::React => "apps",
            ProjectType::DurableObject => "workers",
            ProjectType::Crate => "crates",
        }
    }
}

// A project addition operation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by property tests
pub struct ProjectAdd {
    pub project_type: ProjectType,
    pub name: String,
}
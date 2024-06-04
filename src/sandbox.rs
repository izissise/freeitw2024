use anyhow::Result;
use std::fs;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use tokio::process::{Child, Command};

use enum_dispatch::enum_dispatch;
use serde::Serialize;

/// Kind of sandbox to isolate code
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Clone)]
#[enum_dispatch]
pub enum SandboxKind {
    /// Host wrapper
    Host(Host),
    /// Bwrap wrapper
    BubbleWrap(BubbleWrap),
}

/// Trait to implement sandboxes
#[enum_dispatch(SandboxKind)]
pub trait Trait {
    /// Spawn in the sandbox
    /// # Errors
    ///     Command errors
    fn spawn(&self, prg: &str, params: &[&str]) -> Result<Child>;
    /// Copy resource in the sandbox
    /// # Errors
    ///     IO errors
    fn injest(&self, content: &[u8], filename: &str) -> Result<()>;
}

/// A no sandbox sandbox
#[derive(Serialize, Clone)]
pub struct Host(pub String);

impl Trait for Host {
    fn spawn(&self, prg: &str, params: &[&str]) -> Result<Child> {
        Ok(Command::new(self.0.clone() + "/" + prg).args(params).current_dir(&self.0).spawn()?)
    }

    fn injest(&self, content: &[u8], filename: &str) -> Result<()> {
        let path = self.0.clone() + "/" + filename;
        let mut file = fs::File::create(&path)?;
        file.write_all(content)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}

/// Bwrap sandbox
#[derive(Serialize, Clone)]
pub struct BubbleWrap {
    path: String,
    options: Vec<String>,
}

impl BubbleWrap {
    /// Create a bwrap env
    #[must_use]
    pub fn new<S: Into<String>>(path: S, options: Vec<String>) -> Self {
        let path = path.into();
        Self { path, options }
    }
}

impl Trait for BubbleWrap {
    fn spawn(&self, prg: &str, params: &[&str]) -> Result<Child> {
        if params.is_empty() {
            return Err(anyhow::anyhow!(" Need at least one parameter "));
        }
        Ok(Command::new("/usr/bin/bwrap")
            .args(["--bind", self.path.as_str(), "/wd"])
            .args(&self.options)
            .args(["--"])
            .args(["/wd/".to_string() + prg])
            .args(params)
            .spawn()?)
    }

    fn injest(&self, content: &[u8], filename: &str) -> Result<()> {
        let path = self.path.clone() + "/" + filename;
        let mut file = fs::File::create(&path)?;
        file.write_all(content)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}

use anyhow::Result;
use std::fs;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use tokio::process::Command;

use enum_dispatch::enum_dispatch;
use serde::Serialize;

// TODO add docker

/// Kind of sandbox to isolate code
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Debug)]
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
    fn prepare_spawn(&self, prg: &str) -> Command;
    /// Copy resource in the sandbox
    /// # Errors
    ///     IO errors
    fn injest(&self, content: &[u8], filename: &str) -> Result<()>;
}

/// A no sandbox sandbox
#[derive(Serialize, Debug)]
pub struct Host(pub String);

impl Trait for Host {
    fn prepare_spawn(&self, prg: &str) -> Command {
        let mut cmd = Command::new(self.0.clone() + "/" + prg);
        let _ = cmd.current_dir(&self.0);
        cmd
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
#[derive(Serialize, Debug)]
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
    fn prepare_spawn(&self, prg: &str) -> Command {
        let mut cmd = Command::new("/usr/bin/bwrap");
        let _ = cmd
            .args(["--bind", self.path.as_str(), self.path.as_str()])
            .args(&self.options)
            .args(["--"])
            .args([self.path.to_string() + prg]);
        cmd
    }

    fn injest(&self, content: &[u8], filename: &str) -> Result<()> {
        let path = self.path.clone() + "/" + filename;
        let mut file = fs::File::create(&path)?;
        file.write_all(content)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}

pub fn default_sandboxs(host_wd: String, bwrap_wd: String) -> (Host, BubbleWrap) {
    let host_sb = Host(host_wd);
    let bwrap_sb = BubbleWrap::new(
        bwrap_wd,
        [
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/bin",
            "/bin",
            "--ro-bind",
            "/etc/alternatives",
            "/etc/alternatives",
            "/app",
            "--ro-bind",
            "/etc/ssl/certs",
            "/etc/ssl/certs",
            "--ro-bind",
            "/usr/share/ca-certificates",
            "/usr/share/ca-certificates",
            "--ro-bind",
            "/etc/resolv.conf",
            "/etc/resolv.conf",
            "--ro-bind",
            "/run/systemd/resolve/stub-resolv.conf",
            "/run/systemd/resolve/stub-resolv.conf",
            "--ro-bind",
            "/etc/machine-id",
            "/etc/machine-id",
            "--dev",
            "/dev",
            "--proc",
            "/proc",
            "--tmpfs",
            "/tmp",
            "--unshare-all",
            "--share-net",
            "--hostname",
            "RESTRICTED",
            "--die-with-parent",
            "--new-session",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );
    (host_sb, bwrap_sb)
}

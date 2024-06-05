use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::Stdio;
use tokio::process::Child;

use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use crate::SandboxTrait;

/// Kind of lambda app for now Python or Bash
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[enum_dispatch]
pub enum LambdaAppKind {
    /// Python wrapper
    Py(PyApp),
    /// Bash wrapper
    Bash(BashApp),
}

/// Lambda App trait implement spawn to spawnute the lambda kind
#[enum_dispatch(LambdaAppKind)]
pub trait Trait {
    /// Execute lambda
    /// # Errors
    ///     when Child spawn failed
    fn spawn(
        &self,
        sandbox: &impl SandboxTrait,
        params: &[&str],
        stdin: Stdio,
        stdout: Stdio,
        stderr: Stdio,
    ) -> Result<Child>;
}

/// A python lambda
#[derive(Serialize, Deserialize)]
pub struct PyApp {
    pycode: String,
    entrypoint: String,
}

impl Trait for PyApp {
    fn spawn(
        &self,
        sandbox: &impl SandboxTrait,
        params: &[&str],
        stdin: Stdio,
        stdout: Stdio,
        stderr: Stdio,
    ) -> Result<Child> {
        // make sure it has shebang
        let pycode = "#!/bin/env python3\n".to_string() + &self.pycode;

        // create file unique name
        let mut hasher = DefaultHasher::new();
        pycode.hash(&mut hasher);
        let hash_value = hasher.finish();
        let pname = hash_value.to_string() + ".py";
        sandbox.injest(pycode.as_bytes(), &pname)?;

        // spawn
        Ok(sandbox
            .prepare_spawn(&pname)
            .args(params)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?)
    }
}

/// A bash lambda
#[derive(Serialize, Deserialize)]
pub struct BashApp {
    script: String,
}

impl BashApp {
    /// Create a new `BashApp`
    #[must_use]
    pub fn new<S: Into<String>>(script: S) -> Self {
        let script = script.into();
        Self { script }
    }
}

impl Trait for BashApp {
    fn spawn(
        &self,
        sandbox: &impl SandboxTrait,
        params: &[&str],
        stdin: Stdio,
        stdout: Stdio,
        stderr: Stdio,
    ) -> Result<Child> {
        // make sure it has shebang
        let script = "#!/bin/env bash\n".to_string() + &self.script;

        // create file unique name
        let mut hasher = DefaultHasher::new();
        script.hash(&mut hasher);
        let hash_value = hasher.finish();
        let pname = hash_value.to_string() + ".bash";
        sandbox.injest(script.as_bytes(), &pname)?;

        // spawn
        Ok(sandbox
            .prepare_spawn(&pname)
            .args(params)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?)
    }
}

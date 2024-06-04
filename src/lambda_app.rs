use anyhow::Result;
use tokio::process::Child;

use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use crate::{Sandbox, SandboxTrait};

/// Kind of lambda app for now Python or Bash
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Clone)]
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
    fn spawn(&self, sandbox: &impl SandboxTrait, params: &[&str]) -> Result<Child>;
}

/// A python lambda
#[derive(Serialize, Deserialize, Clone)]
pub struct PyApp {
    pycode: Vec<u8>,
    entrypoint: Vec<u8>,
}

impl Trait for PyApp {
    fn spawn(&self, _sandbox: &impl SandboxTrait, _params: &[&str]) -> Result<Child> {
        unimplemented!()
    }
}

/// A bash lambda
#[derive(Serialize, Deserialize, Clone)]
pub struct BashApp {
    script: Vec<u8>,
}

impl BashApp {
    /// Create a new `BashApp`
    #[must_use]
    pub fn new(script: Vec<u8>) -> Self {
        Self { script }
    }
}

impl Trait for BashApp {
    fn spawn(&self, _sandbox: &impl SandboxTrait, _params: &[&str]) -> Result<Child> {
        unimplemented!()
    }
}

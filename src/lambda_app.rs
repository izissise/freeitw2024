use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use enum_dispatch::enum_dispatch;

use crate::Sandbox;

/// Kind of lambda app for now Python or Bash
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize)]
#[enum_dispatch]
pub enum LambdaAppKind {
    /// Python wrapper
    Py(PyApp),
    /// Bash wrapper
    Bash(BashApp),
}

/// Lambda App trait implement exec to execute the lambda kind
#[allow(async_fn_in_trait)]
#[enum_dispatch(LambdaAppKind)]
pub trait Trait {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8>;
}

/// A python lambda
#[derive(Serialize, Deserialize)]
pub struct PyApp {
    pycode: Vec<u8>,
    entrypoint: Vec<u8>,
}

impl PyApp {
    fn new(pycode: Vec<u8>, entrypoint: Vec<u8>) -> Self {
        Self { pycode, entrypoint }
    }
}

impl Trait for PyApp {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8> {
        unimplemented!()
    }
}

/// A bash lambda
#[derive(Serialize, Deserialize)]
pub struct BashApp {
    script: Vec<u8>,
}

impl BashApp {
    fn new(script: Vec<u8>) -> Self {
        Self { script }
    }
}

impl Trait for BashApp {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8> {
        unimplemented!()
    }
}

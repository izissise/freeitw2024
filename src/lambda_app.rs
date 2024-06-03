use std::collections::HashMap;

use enum_dispatch::enum_dispatch;

use crate::sandbox::Sandbox;

#[enum_dispatch]
pub enum LambdaApp {
    Py(PyApp),
    Bash(BashApp),
}

#[enum_dispatch(LambdaApp)]
pub trait LambdaAppTrait {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8>;
}

struct PyApp {
    pycode: Vec<u8>,
    entrypoint: Vec<u8>,
}

impl PyApp {
    fn new(pycode: Vec<u8>, entrypoint: Vec<u8>) -> Self {
        Self { pycode, entrypoint }
    }
}

impl LambdaAppTrait for PyApp {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8> {
        unimplemented!()
    }
}

struct BashApp {
    script: Vec<u8>,
}

impl BashApp {
    fn new(script: Vec<u8>) -> Self {
        Self { script }
    }
}

impl LambdaAppTrait for BashApp {
    async fn exec(&self, sandbox: Sandbox, params: HashMap<String, String>) -> Vec<u8> {
        unimplemented!()
    }
}

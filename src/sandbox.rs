use std::collections::HashMap;

use axum::extract::Host;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub enum Sandbox {
    Host(HostSandbox),
    BubbleWrap(BubbleWrapSandbox),
}

#[allow(async_fn_in_trait)]
#[enum_dispatch(Sandbox)]
pub trait SandboxTrait {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8>;
}

struct HostSandbox {
    shell: bool,
}

impl HostSandbox {
    fn new(shell: bool) -> Self {
        Self { shell }
    }
}

impl SandboxTrait for HostSandbox {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8> {
        todo!();
    }
}

struct BubbleWrapSandbox {
    options: Vec<String>,
}

impl BubbleWrapSandbox {
    fn new(options: Vec<String>) -> Self {
        Self { options }
    }
}

impl SandboxTrait for BubbleWrapSandbox {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8> {
        todo!();
    }
}

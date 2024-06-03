use enum_dispatch::enum_dispatch;
use serde::Serialize;
use std::collections::HashMap;

/// Kind of sandbox to isolate code
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize)]
#[enum_dispatch]
pub enum SandboxKind {
    /// Host wrapper
    Host(Host),
    /// Bwrap wrapper
    BubbleWrap(BubbleWrap),
}

/// Trait to implement sandboxes
#[allow(async_fn_in_trait)]
#[enum_dispatch(SandboxKind)]
pub trait Trait {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8>;
}

/// A no sandbox sandbox
#[derive(Serialize)]
pub struct Host {
    /// Use a shell?
    shell: bool,
}

impl Host {
    fn new(shell: bool) -> Self {
        Self { shell }
    }
}

impl Trait for Host {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8> {
        todo!();
    }
}

/// Bwrap sandbox
#[derive(Serialize)]
pub struct BubbleWrap {
    options: Vec<String>,
}

impl BubbleWrap {
    fn new(options: Vec<String>) -> Self {
        Self { options }
    }
}

impl Trait for BubbleWrap {
    async fn exec(&self, params: HashMap<String, String>) -> Vec<u8> {
        todo!();
    }
}

//! Free interview 2024

#![doc = include_str!("../Readme.md")]
// Macro options
#![recursion_limit = "512"]
// Lints
#![warn(unsafe_code)]
#![deny(unused_results)]
#![warn(missing_docs)]
// Clippy lint options
// see clippy.toml
// https://rust-lang.github.io/rust-clippy/master/index.html
#![deny(
    // Pedantic
    clippy::pedantic,
)]
#![warn(
    // Restriction
    clippy::allow_attributes_without_reason,
    clippy::decimal_literal_representation,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::default_union_representation,
    clippy::exit,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::mod_module_files,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::same_name_method,
    clippy::shadow_unrelated,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads,
    clippy::empty_drop,
    clippy::mixed_read_write_in_expression,
    // clippy::pub_use,

    // Nursery
    clippy::cognitive_complexity,
    clippy::debug_assert_with_mut_call,
    clippy::future_not_send,
    clippy::imprecise_flops,

    // Cargo
//     clippy::multiple_crate_versions, // check from time to time
    clippy::wildcard_dependencies,
)]
#![allow(clippy::match_bool)]

use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
use log::info;
use std::process::Stdio;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::prelude::*;

/// Error module
mod error;

/// Lambda app module
mod lambda_app;

/// http Pagination
mod pagination;

/// Sandboxing
mod sandbox;

mod api;

use api::{
    lambda_delete, lambda_exec, lambda_get, lambdas_index, lambdas_insert, sandboxs_index, AppState,
};
use lambda_app::{BashApp, Trait as LambdaTrait};
use sandbox::{
    default_sandboxs, Host as SandboxHost, SandboxKind as Sandbox, Trait as SandboxTrait,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("tower_http::trace::on_response", Level::DEBUG)
                .with_target("tower_http::trace::on_request", Level::DEBUG)
                .with_target("tower_http::trace::make_span", Level::DEBUG)
                .with_default(Level::INFO),
        )
        .init();

    let wd = "/tmp/freeitw_wd".to_string();
    std::fs::create_dir_all(&wd)?;
    let init_host_sb = SandboxHost(wd.clone());
    let init = BashApp::new(
        r#"#!/bin/env bash
set -ex
WD=$1
mkdir -p "$WD"

command -v python3 &>/dev/null || exit 127
command -v pip3 &>/dev/null || exit 127
command -v bwrap &>/dev/null || exit 127

python3 -m venv "$WD"
source "$WD"/bin/activate
pip3 install pandas
    "#,
    );

    let host_wd = wd.clone();
    let bwrap_wd = wd.clone();

    info!("Setup bwrap sandbox...");
    let init =
        init.spawn(&init_host_sb, &[&wd], Stdio::inherit(), Stdio::inherit(), Stdio::inherit())?;
    let out = init.wait_with_output().await?;
    if !out.status.success() {
        return Err(anyhow::anyhow!(out.status));
    }
    let (host_sb, bwrap_sb) = default_sandboxs(host_wd, bwrap_wd);

    let mut sandboxs = HashMap::new();
    let _ = sandboxs.insert("host".to_string(), Arc::new(Sandbox::Host(host_sb)));
    let _ = sandboxs.insert("bwrap".to_string(), Arc::new(Sandbox::BubbleWrap(bwrap_sb)));

    let state = Arc::new(RwLock::new(AppState { lambdas: HashMap::new(), sandboxs }));

    // Compose the routes
    let app = Router::new()
        .route("/sandboxs", get(sandboxs_index))
        .route("/lambdas", get(lambdas_index).put(lambdas_insert))
        .route("/lambdas/:name/exec", post(lambda_exec))
        .route("/lambdas/:name", get(lambda_get).delete(lambda_delete))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("Listening on port 3000");
    let listener = tokio::net::TcpListener::bind(":::3000").await?;
    Ok(axum::serve(listener, app).await?)
}

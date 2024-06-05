//! Free interview 2024

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
use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result as HttpResult},
    routing::{get, post},
    Json, Router,
};
use futures::TryStreamExt;
use log::info;
use serde::Deserialize;
use std::process::Stdio;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::{
    io::{copy, AsyncReadExt, BufReader},
    select,
};
use tokio_util::io::StreamReader;
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

use error::HttpErr;
use lambda_app::{BashApp, LambdaAppKind as LambdaApp, Trait as LambdaTrait};
use pagination::Pagination;
use sandbox::{
    BubbleWrap as SandboxBubbleWrap, Host as SandboxHost, SandboxKind as Sandbox,
    Trait as SandboxTrait,
};

struct ApiState {
    lambdas: HashMap<String, Arc<LambdaApp>>,
    sandboxs: HashMap<String, Arc<Sandbox>>,
}

type ApiStateWrapper = Arc<RwLock<ApiState>>;
type HttpResponse = HttpResult<Response<Body>, HttpErr>;

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
mkdir -p "$WD"/host "$WD"/bwrap

command -v python3 &>/dev/null || exit 127
command -v pip3 &>/dev/null || exit 127
command -v bwrap &>/dev/null || exit 127

python3 -m venv "$WD"/bwrap
source "$WD"/bwrap/bin/activate
pip3 install panda
    "#,
    );

    let host_wd = wd.clone() + "/host";
    let bwrap_wd = wd.clone() + "/bwrap";

    info!("Setup bwrap sandbox...");
    let init =
        init.spawn(&init_host_sb, &[&wd], Stdio::inherit(), Stdio::inherit(), Stdio::inherit())?;
    let out = init.wait_with_output().await?;
    if !out.status.success() {
        return Err(anyhow::anyhow!(out.status));
    }
    let bwrap_sb = SandboxBubbleWrap::new(
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

    let host_sb = SandboxHost(host_wd);
    let mut sandboxs = HashMap::new();
    let _ = sandboxs.insert("host".to_string(), Arc::new(Sandbox::Host(host_sb)));
    let _ = sandboxs.insert("bwrap".to_string(), Arc::new(Sandbox::BubbleWrap(bwrap_sb)));

    let state = Arc::new(RwLock::new(ApiState { lambdas: HashMap::new(), sandboxs }));

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

/// Return state locked for reading
fn lock_state_read(state: &ApiStateWrapper) -> Result<std::sync::RwLockReadGuard<'_, ApiState>> {
    // With map errors to string because PoisonError are not `Send`
    state.read().map_err(move |e| anyhow::anyhow! { e.to_string() })
}

/// Return state locked for writing
fn lock_state_write(state: &ApiStateWrapper) -> Result<std::sync::RwLockWriteGuard<'_, ApiState>> {
    // With map errors to string because PoisonError are not `Send`
    state.write().map_err(move |e| anyhow::anyhow! { e.to_string() })
}

async fn sandboxs_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> HttpResponse {
    let Query(pagination) = pagination.unwrap_or_default();

    let state = lock_state_read(&s)?;
    let sandboxs: HashMap<_, _> =
        state.sandboxs.iter().skip(pagination.offset).take(pagination.limit).collect();

    Ok(Json(sandboxs).into_response())
}

async fn lambdas_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> HttpResponse {
    let Query(pagination) = pagination.unwrap_or_default();

    let state = lock_state_read(&s)?;
    let lambdas: HashMap<_, _> =
        state.lambdas.iter().skip(pagination.offset).take(pagination.limit).collect();

    Ok(Json(lambdas).into_response())
}

#[derive(Deserialize)]
struct LambdasInsert {
    name: String,
    #[serde(flatten)]
    app: LambdaApp,
}

async fn lambdas_insert(
    State(s): State<ApiStateWrapper>,
    lambdasinsert: Json<LambdasInsert>,
) -> HttpResponse {
    let lambdasinsert = lambdasinsert.0;

    let mut state = lock_state_write(&s)?;
    let _ = state.lambdas.insert(lambdasinsert.name, Arc::new(lambdasinsert.app));

    Ok(StatusCode::CREATED.into_response())
}

async fn lambda_get(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let state = lock_state_read(&s)?;
    let lambda = state.lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(lambda).into_response())
}

async fn lambda_delete(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let mut state = lock_state_write(&s)?;
    let _ = state.lambdas.remove(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, Deserialize)]
struct ExecParams {
    sandbox: String,
    args: String,
    status: bool,
}

impl Default for ExecParams {
    fn default() -> Self {
        Self { sandbox: "host".to_string(), args: String::new(), status: false }
    }
}

async fn lambda_exec(
    params: Option<Query<ExecParams>>,
    Path(name): Path<String>,
    State(s): State<ApiStateWrapper>,
    req: Request,
) -> HttpResponse {
    // Url query parameters
    let Query(params) = params.unwrap_or_default();
    let sandbox = params.sandbox;
    let args = params.args.split_whitespace().collect::<Vec<_>>();
    let print_status = params.status;

    // Convert the body into an `AsyncRead`.
    let body = req.into_body().into_data_stream();
    let body_with_io_error =
        body.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Here we need to retrieve and drop the state lock
    // because ReadLockGuard is !Send and so we cannot keep it across an await point
    // (it would need to be locked and unlocked on the same thread during child wait() which tokio doesn't guarantee)
    // Since our state uses Arc, clone is just a ptr copy
    let (lambda, sandbox) = {
        let state = lock_state_read(&s)?;
        (
            Arc::clone(state.lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?),
            Arc::clone(state.sandboxs.get(&sandbox).ok_or(StatusCode::NOT_FOUND)?),
        )
    };

    // SPAWN THE CHILD PROCESS
    let mut child =
        lambda.spawn(&*sandbox, &args, Stdio::piped(), Stdio::piped(), Stdio::piped())?;

    let mut stdin = child.stdin.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let stdout = BufReader::new(child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?);
    let stderr = BufReader::new(child.stderr.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?);
    futures::pin_mut!(stdout);
    futures::pin_mut!(stderr);
    let mut stdout_buf = vec![0_u8; 128];
    let mut stderr_buf = vec![0_u8; 128];

    let mut child_alive = true;
    while child_alive {
        select! {
            _ = copy(&mut body_reader, &mut stdin) => {
                // Copy body to process stdin,
            },
            status = child.wait() => {
                let status = status?;
                if print_status {
                    // TODO add final text with status
                }
                dbg!(status);
                child_alive = false;
            },
            n = stdout.read(&mut stdout_buf) => {
                let n = n?;
                let b = &stdout_buf[..n];
                println!("{}", String::from_utf8(b.to_vec()).unwrap());
                // TODO stream to response
            }
            n = stderr.read(&mut stderr_buf) => {
                let n = n?;
                let b = &stderr_buf[..n];
                println!("{}", String::from_utf8(b.to_vec()).unwrap());
                // TODO stream to response
            }
        }
    }

    Ok(StatusCode::OK.into_response())
}

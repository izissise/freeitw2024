use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result as HttpResult},
    routing::{get, post},
    Json, Router,
};
use log::*;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::prelude::*;

use interviewfree::{
    BashApp, HttpErr, LambdaApp, LambdaTrait, Pagination, Sandbox, SandboxBubbleWrap, SandboxHost,
};

struct ApiState {
    lambdas: HashMap<String, LambdaApp>,
    sandboxs: HashMap<String, Sandbox>,
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
    let host_sb = SandboxHost(wd.clone());
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
    let init = init.spawn(&host_sb, &[&wd])?;
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
    let _ = sandboxs.insert("host".to_string(), Sandbox::Host(host_sb));
    let _ = sandboxs.insert("bwrap".to_string(), Sandbox::BubbleWrap(bwrap_sb));

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
    let state = lock_state_read(&s)?;
    let sandboxs = &state.sandboxs;

    let Query(pagination) = pagination.unwrap_or_default();

    let sandboxs: HashMap<&String, &Sandbox> =
        sandboxs.iter().skip(pagination.offset).take(pagination.limit).collect();

    Ok(Json(sandboxs).into_response())
}

async fn lambdas_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> HttpResponse {
    let state = lock_state_read(&s)?;
    let lambdas = &state.lambdas;

    let Query(pagination) = pagination.unwrap_or_default();

    let lambdas: HashMap<&String, &LambdaApp> =
        lambdas.iter().skip(pagination.offset).take(pagination.limit).collect();

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
    let lambdas = &mut state.lambdas;
    let _ = lambdas.insert(lambdasinsert.name, lambdasinsert.app);
    Ok(StatusCode::CREATED.into_response())
}

async fn lambda_get(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let state = lock_state_read(&s)?;
    let lambdas = &state.lambdas;
    let lambda = lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(lambda).into_response())
}

async fn lambda_delete(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let mut state = lock_state_write(&s)?;
    let lambdas = &mut state.lambdas;
    lambdas.remove(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK.into_response())
}

async fn lambda_exec(
    Path(name): Path<String>,
    State(s): State<ApiStateWrapper>,
    _params: Json<serde_json::Value>,
) -> HttpResponse {
    // Here we need to retrieve and drop the lambda definition
    // because ReadLockGuard is !Send and so we cannot keep it across an await point
    // (it would need to be locked and unlocked on the same thread during child wait() which tokio doesn't guarantee)
    let mut child = {
        let state = lock_state_read(&s)?;
        let lambdas = &state.lambdas;
        let sandboxs = &state.sandboxs;
        let lambda = lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?;
        // TODO choose sandbox from header
        let sandbox = sandboxs.get("host").ok_or(StatusCode::NOT_FOUND)?;
        lambda.spawn(sandbox, &[])?
    };

    let _status = child.wait().await.unwrap(); // FIXME

    Ok(StatusCode::OK.into_response())
}

use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result as HttpResult},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;

use interviewfree::{
    HttpErr, LambdaApp, LambdaTrait, Pagination, Sandbox, SandboxBubbleWrap, SandboxHost,
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
    tracing_subscriber::fmt::init();

    // TODO create bubblewrap sandbox

    let state =
        Arc::new(RwLock::new(ApiState { lambdas: HashMap::new(), sandboxs: HashMap::new() }));

    // Compose the routes
    let app = Router::new()
        .route("/sandboxs", get(sandboxs_index))
        .route("/lambdas", get(lambdas_index).put(lambdas_insert))
        .route("/lambdas/:name/exec", post(lambda_exec))
        .route("/lambdas/:name", get(lambda_get).delete(lambda_delete))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

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

    let sandboxs =
        sandboxs.values().skip(pagination.offset).take(pagination.limit).collect::<Vec<_>>();

    Ok(Json(sandboxs).into_response())
}

async fn lambdas_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> HttpResponse {
    let state = lock_state_read(&s)?;
    let lambdas = &state.lambdas;

    let Query(pagination) = pagination.unwrap_or_default();

    let lambdas =
        lambdas.values().skip(pagination.offset).take(pagination.limit).collect::<Vec<_>>();

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
    // Here we need to retrieve and clone the lambda definition
    // because ReadLockGuard is !Send and so we cannot keep it across an await point
    // (it would need to be locked and unlocked on the same thread which tokio doesn't guarantee)
    // FIXME without cloning?
    let lambda = {
        let state = lock_state_read(&s)?;
        let lambdas = &state.lambdas;
        lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?.clone()
    };

    let res = lambda.exec(SandboxHost::new(true), HashMap::new()).await?;

    Ok(res.into_response())
}

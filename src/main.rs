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

async fn sandboxs_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> HttpResponse {
    let state = s.read().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
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
    let state = s.read().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
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

    let mut state = s.write().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
    let lambdas = &mut state.lambdas;
    let _ = lambdas.insert(lambdasinsert.name, lambdasinsert.app);
    Ok(StatusCode::CREATED.into_response())
}

async fn lambda_get(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let state = s.read().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
    let lambdas = &state.lambdas;
    let lambda = lambdas.get(&name).ok_or_else(|| anyhow::anyhow! { "Not found" })?; // TODO fix

    Ok(Json(lambda).into_response())
}

async fn lambda_delete(Path(name): Path<String>, State(s): State<ApiStateWrapper>) -> HttpResponse {
    let mut state = s.write().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
    let lambdas = &mut state.lambdas;
    lambdas.remove(&name).ok_or_else(|| anyhow::anyhow! { "Not found" })?; // TODO fix

    Ok(StatusCode::OK.into_response())
}

async fn lambda_exec(
    Path(name): Path<String>,
    State(s): State<ApiStateWrapper>,
    _params: Json<serde_json::Value>,
) -> HttpResponse {
    let state = s.read().map_err(|e| anyhow::anyhow! { e.to_string() })?; // With map errors this way because PoisonError are not `Send`
    let lambdas = &state.lambdas;
    let lambda = lambdas.get(&name).ok_or_else(|| anyhow::anyhow! { "Not found" })?; // TODO fix

    // FIXME the await in the following provoke the compile error
    // let res = lambda.exec(Sandbox::Host(SandboxHost::new(true)), HashMap::new()).await; // TODO params

    Ok(StatusCode::OK.into_response())
}

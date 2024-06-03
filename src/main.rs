use anyhow::Result;
use axum::{
    body::Body,
    extract::{Query, State},
    response::{IntoResponse, Response, Result as HttpResult},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;

use interviewfree::{HttpErr, LambdaApp, Pagination, Sandbox};

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

    let state =
        Arc::new(RwLock::new(ApiState { lambdas: HashMap::new(), sandboxs: HashMap::new() }));

    // Compose the routes
    let app = Router::new()
        .route("/sandboxs", get(sandboxs_index))
        .route("/lambdas", get(lambdas_index).put(lambdas_insert))
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
    Ok("".into_response())
}

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
    Json, Router,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio;
use tracing_subscriber;
use tower_http::trace::TraceLayer;
use serde::Deserialize;

use interviewfree::{ApiError, Pagination, LambdaApp, Sandbox};

struct ApiState {
    apps: HashMap<String, LambdaApp>,
    sandboxs: HashMap<String, Sandbox>,
}

type ApiStateWrapper = Arc<RwLock<ApiState>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    tracing_subscriber::fmt::init();

    let state = Arc::new(RwLock::new(ApiState { apps: HashMap::new(), sandboxs: HashMap::new() }));

    // Compose the routes
    let app = Router::new()
        .route("/apps",
               get(apps_index)
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(":::3000").await?;
    Ok(axum::serve(listener, app).await?)
}

async fn apps_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<ApiStateWrapper>,
) -> impl IntoResponse {
    let state = s.read().unwrap(); // TODO handle error
    let apps = &state.apps;

    let Query(pagination) = pagination.unwrap_or_default();

    let apps = apps
        .values()
        .skip(pagination.offset)
        .take(pagination.limit)
        .collect::<Vec<_>>();

    Json(apps).into_response()
}


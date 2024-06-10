use crate::{error::HttpErr, pagination::Pagination};
use anyhow::Result;
use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result as HttpResult},
    Json,
};
use axum_extra::body::AsyncReadBody;
use futures::TryStreamExt;
use serde::Deserialize;
use std::process::Stdio;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    select,
    sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::StreamReader;

pub struct AppState {
    /// Lambdas container
    pub lambdas: HashMap<String, Arc<LambdaApp>>,
    /// Sandboxes container
    pub sandboxs: HashMap<String, Arc<Sandbox>>,
}
use crate::lambda_app::{LambdaAppKind as LambdaApp, Trait as LambdaTrait};
use crate::sandbox::SandboxKind as Sandbox;

pub type AppStateWrapper = Arc<RwLock<AppState>>;
pub type HttpResponse = HttpResult<Response<Body>, HttpErr>;

/// Return state locked for reading
fn lock_state_read(state: &AppStateWrapper) -> Result<std::sync::RwLockReadGuard<'_, AppState>> {
    // With map errors to string because PoisonError are not `Send`
    state.read().map_err(move |e| anyhow::anyhow! { e.to_string() })
}

/// Return state locked for writing
fn lock_state_write(state: &AppStateWrapper) -> Result<std::sync::RwLockWriteGuard<'_, AppState>> {
    // With map errors to string because PoisonError are not `Send`
    state.write().map_err(move |e| anyhow::anyhow! { e.to_string() })
}

/// Handler to return a paginated list of sandboxes
pub async fn sandboxs_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<AppStateWrapper>,
) -> HttpResponse {
    let Query(pagination) = pagination.unwrap_or_default();

    let state = lock_state_read(&s)?;
    let sandboxs: HashMap<_, _> =
        state.sandboxs.iter().skip(pagination.offset).take(pagination.limit).collect();

    Ok(Json(sandboxs).into_response())
}

/// Handler to return a paginated list of lambda applications
pub async fn lambdas_index(
    pagination: Option<Query<Pagination>>,
    State(s): State<AppStateWrapper>,
) -> HttpResponse {
    let Query(pagination) = pagination.unwrap_or_default();

    let state = lock_state_read(&s)?;
    let lambdas: HashMap<_, _> =
        state.lambdas.iter().skip(pagination.offset).take(pagination.limit).collect();

    Ok(Json(lambdas).into_response())
}

/// Structure to receive data for creating a new lambda
#[derive(Deserialize)]
pub struct LambdasInsert {
    name: String,
    #[serde(flatten)]
    app: LambdaApp,
}

/// Handler to insert a new lambda application
pub async fn lambdas_insert(
    State(s): State<AppStateWrapper>,
    lambdasinsert: Json<LambdasInsert>,
) -> HttpResponse {
    let lambdasinsert = lambdasinsert.0;

    let mut state = lock_state_write(&s)?;
    let _ = state.lambdas.insert(lambdasinsert.name, Arc::new(lambdasinsert.app));

    Ok(StatusCode::CREATED.into_response())
}

/// Handler to retrieve a lambda application by name
pub async fn lambda_get(
    Path(name): Path<String>,
    State(s): State<AppStateWrapper>,
) -> HttpResponse {
    let state = lock_state_read(&s)?;
    let lambda = state.lambdas.get(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(lambda).into_response())
}

/// Handler to delete a lambda application by name
pub async fn lambda_delete(
    Path(name): Path<String>,
    State(s): State<AppStateWrapper>,
) -> HttpResponse {
    let mut state = lock_state_write(&s)?;
    let _ = state.lambdas.remove(&name).ok_or(StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK.into_response())
}

/// Lambda execution parameters
#[derive(Debug, Deserialize)]
pub struct ExecParams {
    sandbox: String,
    args: String,
    status: bool,
}

impl Default for ExecParams {
    fn default() -> Self {
        Self { sandbox: "host".to_string(), args: String::new(), status: false }
    }
}

/// Handler to execute a lambda function
pub async fn lambda_exec(
    params: Option<Query<ExecParams>>,
    Path(name): Path<String>,
    State(s): State<AppStateWrapper>,
    req: Request,
) -> HttpResponse {
    // Url query parameters
    let Query(params) = params.unwrap_or_default();
    let sandbox = params.sandbox;
    let args = params.args.split_whitespace().collect::<Vec<_>>();
    let print_status = params.status;

    // Convert the body into an `AsyncRead`.
    let body = req.into_body().into_data_stream().map_err(std::io::Error::other);
    let body_with_io_error =
        body.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);

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

    // setup streaming
    let stdin = child.stdin.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let stdout = BufReader::new(child.stdout.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?);
    let stderr = BufReader::new(child.stderr.take().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?);

    let mut body_reader = Box::pin(body_reader);
    let mut stdout = Box::pin(stdout);
    let mut stderr = Box::pin(stderr);
    let mut stdin_buf = vec![0_u8; 128];
    let mut stdout_buf = vec![0_u8; 128];
    let mut stderr_buf = vec![0_u8; 128];

    let (tx, rx) = mpsc::channel::<Result<Bytes, HttpErr>>(4);

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(async move {
        let mut stdin_container = Some(stdin);
        loop {
            select! {
                status = child.wait() => {
                    let status = match status {
                        Err(e) => return tx.send(Err(HttpErr::Io(e))).await.expect("channel to be alive"),
                        Ok(status) => status,
                    };
                   if print_status {
                       let () = tx.send(Ok(Bytes::from(format!("Exit status {status}")))).await.expect("channel to be alive");
                   }
                   break;
               },
               n = body_reader.read(&mut stdin_buf) => {
                    let n = match n {
                        Err(e) => return tx.send(Err(HttpErr::Io(e))).await.expect("channel to be alive"),
                        Ok(n) => n,
                    };
                    if n == 0 {
                        if let Some(child_stdin) = stdin_container.take() {
                            // Drop stdin
                            std::mem::drop(child_stdin);
                        }
                    }
                    if let Some(child_stdin) = stdin_container.as_mut() {
                        let _err = child_stdin.write_all(&stdin_buf[..n]).await.is_err();
                    }
               }
               n = stdout.read(&mut stdout_buf) => {
                    let n = match n {
                        Err(e) => return tx.send(Err(HttpErr::Io(e))).await.expect("channel to be alive"),
                        Ok(n) => n,
                    };

                   let b = stdout_buf[..n].to_vec();
                   let () = tx.send(Ok(Bytes::from(b))).await.expect("channel to be alive");
               }
               n = stderr.read(&mut stderr_buf) => {
                    let n = match n {
                        Err(e) => return tx.send(Err(HttpErr::Io(e))).await.expect("channel to be alive"),
                        Ok(n) => n,
                    };
                    let b = stderr_buf[..n].to_vec();
                    let () = tx.send(Ok(Bytes::from(b))).await.expect("channel to be alive");
                }
            }
        }
    });

    Ok((StatusCode::OK, AsyncReadBody::new(StreamReader::new(ReceiverStream::new(rx))))
        .into_response())
}

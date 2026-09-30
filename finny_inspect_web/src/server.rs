//! The HTTP API and the frontend.

use std::{convert::Infallible, io, net::{SocketAddr, ToSocketAddrs}, sync::Arc};

use axum::{Json, Router, extract::{Path, Query, State}, http::{HeaderMap, StatusCode, header}, response::{IntoResponse, Response, sse::{Event, KeepAlive, Sse}}, routing::get};
use futures_util::{Stream, StreamExt, stream};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::{Inspector, assets, registry::{FsmInstance, Registry, StoredSnapshot}};

type AppState = Arc<Registry>;

impl Inspector {
    /// The frontend and its API, to be served or nested into an existing axum application.
    ///
    /// * `GET /` - the frontend
    /// * `GET /api/instances` - the attached FSM instances
    /// * `GET /api/instances/stream` - server-sent events, `instances`, with the list of the
    ///   instances whenever it changes
    /// * `GET /api/instances/{id}/meta` - the FSM's description, `{ info, plantuml }`
    /// * `GET /api/instances/{id}/snapshots?after={seq}` - the kept snapshots
    /// * `GET /api/instances/{id}/stream?after={seq}` - server-sent events, `snapshot`, with
    ///   the kept snapshots after `seq` (or the `Last-Event-ID`) followed by the new ones. A
    ///   `reset` event asks the client to reload the snapshots, it fell behind.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(index))
            .route("/assets/{*path}", get(asset))
            .route("/api/instances", get(instances))
            .route("/api/instances/stream", get(instances_stream))
            .route("/api/instances/{id}/meta", get(meta))
            .route("/api/instances/{id}/snapshots", get(snapshots))
            .route("/api/instances/{id}/stream", get(snapshots_stream))
            .with_state(self.registry.clone())
    }

    /// Serves the frontend on the current tokio runtime, until the listener fails.
    pub async fn serve(&self, addr: impl tokio::net::ToSocketAddrs) -> io::Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router()).await
    }

    /// Serves the frontend on a background thread with its own runtime, for applications with
    /// synchronous FSMs or without tokio. Returns the bound address, useful with port 0.
    pub fn spawn(&self, addr: impl ToSocketAddrs) -> io::Result<SocketAddr> {
        let listener = std::net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let local_addr = listener.local_addr()?;
        let router = self.router();

        std::thread::Builder::new()
            .name("finny-inspect-web".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to start the inspector's runtime");
                        return;
                    }
                };

                runtime.block_on(async move {
                    let result = match tokio::net::TcpListener::from_std(listener) {
                        Ok(listener) => axum::serve(listener, router).await,
                        Err(e) => Err(e)
                    };
                    if let Err(e) = result {
                        tracing::error!(error = %e, "The inspector's server failed");
                    }
                });
            })?;

        Ok(local_addr)
    }
}

async fn index(State(registry): State<AppState>, headers: HeaderMap) -> Response {
    assets::serve(&registry.config, "index.html", &headers)
}

async fn asset(State(registry): State<AppState>, Path(path): Path<String>, headers: HeaderMap) -> Response {
    assets::serve(&registry.config, &path, &headers)
}

async fn instances(State(registry): State<AppState>) -> impl IntoResponse {
    Json(registry.summaries())
}

async fn instances_stream(State(registry): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = registry.changes.subscribe();
    let list = move |registry: &Registry| {
        Ok(Event::default().event("instances").data(serde_json::to_string(&registry.summaries()).unwrap_or_default()))
    };

    let first = stream::once(std::future::ready(list(&registry)));
    let changes = stream::unfold((registry, rx), move |(registry, mut rx)| async move {
        match rx.recv().await {
            Ok(()) | Err(RecvError::Lagged(_)) => {
                let event = list(&registry);
                Some((event, (registry, rx)))
            },
            Err(RecvError::Closed) => None
        }
    });

    Sse::new(first.chain(changes)).keep_alive(KeepAlive::default())
}

fn find(registry: &Registry, id: &str) -> Result<Arc<FsmInstance>, Response> {
    registry.instance(id).ok_or_else(|| (StatusCode::NOT_FOUND, format!("Unknown FSM instance '{}'", id)).into_response())
}

fn json_response(body: impl Into<axum::body::Body>) -> Response {
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-store")], body.into()).into_response()
}

async fn meta(State(registry): State<AppState>, Path(id): Path<String>) -> Result<Response, Response> {
    let instance = find(&registry, &id)?;
    Ok(json_response(instance.meta_json.to_string()))
}

#[derive(Deserialize)]
struct AfterQuery {
    after: Option<u64>
}

async fn snapshots(State(registry): State<AppState>, Path(id): Path<String>, Query(q): Query<AfterQuery>) -> Result<Response, Response> {
    let instance = find(&registry, &id)?;
    // the stored JSON is reused, not serialized again
    let body = {
        let state = instance.lock_state();
        let mut body = String::from("[");
        for (i, s) in state.ring.iter().filter(|s| Some(s.seq) > q.after).enumerate() {
            if i > 0 {
                body.push(',');
            }
            body.push_str(&s.json);
        }
        body.push(']');
        body
    };
    Ok(json_response(body))
}

fn snapshot_event(s: &StoredSnapshot) -> Event {
    Event::default().event("snapshot").id(s.seq.to_string()).data(&*s.json)
}

async fn snapshots_stream(State(registry): State<AppState>, Path(id): Path<String>, Query(q): Query<AfterQuery>, headers: HeaderMap)
    -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response>
{
    let instance = find(&registry, &id)?;

    // a reconnecting EventSource continues after the last received snapshot
    let last_event_id = headers.get("last-event-id").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
    let (backlog, rx) = instance.subscribe(last_event_id.or(q.after));

    let backlog = stream::iter(backlog.into_iter().map(|s| Ok(snapshot_event(&s))));
    let live = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(s) => Some((Ok(snapshot_event(&s)), rx)),
            Err(RecvError::Lagged(n)) => Some((Ok(Event::default().event("reset").data(n.to_string())), rx)),
            Err(RecvError::Closed) => None
        }
    });

    Ok(Sse::new(backlog.chain(live)).keep_alive(KeepAlive::default()))
}

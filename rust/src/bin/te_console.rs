//! Local HTTP console: Vue SPA + JSON simulation / experiment / session API.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use tennessee_eastman::catalog::catalog;
use tennessee_eastman::experiment::{mimo_sim_csv, run as run_experiment, ExperimentRequest};
use tennessee_eastman::session::{PlantSession, SessionConfig, StepResponse};
use tennessee_eastman::simulate::{run, SimulationRequest};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Clone)]
struct AppState {
    dist: PathBuf,
    sessions: Arc<Mutex<HashMap<String, PlantSession>>>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let dist = dist_dir();
    let state = AppState {
        dist: dist.clone(),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut router = Router::new()
        .route("/api/catalog", get(get_catalog))
        .route("/api/simulate", post(post_simulate))
        .route("/api/experiment", post(post_experiment))
        .route("/api/export/mimo-csv", post(post_export_csv))
        .route("/api/session", post(create_session))
        .route("/api/session/{id}/step", post(session_step))
        .route("/api/session/{id}", delete(delete_session));

    router = if dist.join("index.html").is_file() {
        router.fallback_service(
            ServeDir::new(&dist).not_found_service(ServeFile::new(dist.join("index.html"))),
        )
    } else {
        router
            .route("/", get(missing_frontend))
            .fallback(missing_frontend)
    };

    let app = router
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .with_state(state);

    let listener = TcpListener::bind(opts.addr).await.expect("bind");
    let actual = listener.local_addr().expect("local addr");
    eprintln!("TE console  http://{actual}");
    if dist.join("index.html").is_file() {
        eprintln!("serving SPA from {}", dist.display());
    } else {
        eprintln!(
            "no SPA at {} — API only. Build with:  cd web && npm install && npm run build",
            dist.display()
        );
    }
    axum::serve(listener, app).await.expect("server");
}

async fn get_catalog() -> Json<tennessee_eastman::catalog::Catalog> {
    Json(catalog())
}

async fn post_simulate(Json(req): Json<SimulationRequest>) -> Response {
    match tokio::task::spawn_blocking(move || run(&req)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(err)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": err.0})),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

async fn post_experiment(Json(req): Json<ExperimentRequest>) -> Response {
    match tokio::task::spawn_blocking(move || run_experiment(&req)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(err)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": err.0})),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct ExportCsvRequest {
    mv: Vec<Vec<f32>>,
    cv: Vec<Vec<f32>>,
    time_s: Vec<u32>,
    #[serde(default = "default_record_every")]
    record_every: usize,
}

fn default_record_every() -> usize {
    60
}

async fn post_export_csv(Json(req): Json<ExportCsvRequest>) -> Response {
    let csv = mimo_sim_csv(&req.time_s, &req.mv, &req.cv, req.record_every);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"te_export.csv\"",
            ),
        ],
        csv,
    )
        .into_response()
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    snapshot: StepResponse,
}

async fn create_session(
    State(state): State<AppState>,
    Json(cfg): Json<SessionConfig>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = PlantSession::new(&cfg);
    let snapshot = session.snapshot();
    let id = format!("{:x}", rand_u64());
    state
        .sessions
        .lock()
        .expect("sessions")
        .insert(id.clone(), session);
    Ok(Json(CreateSessionResponse {
        session_id: id,
        snapshot,
    }))
}

#[derive(Deserialize)]
struct StepRequest {
    #[serde(default)]
    pub setpoint_writes: std::collections::BTreeMap<usize, f64>,
}

async fn session_step(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<StepRequest>,
) -> Result<Json<StepResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mut guard = state.sessions.lock().expect("sessions");
    let session = guard.get_mut(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "session not found"})),
        )
    })?;
    let resp = session.step_apc(&body.setpoint_writes);
    Ok(Json(resp))
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    state.sessions.lock().expect("sessions").remove(&id);
    StatusCode::NO_CONTENT
}

fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    t as u64 ^ (t >> 64) as u64
}

async fn missing_frontend(State(state): State<AppState>) -> Html<String> {
    Html(format!(
        "<!doctype html><meta charset=utf-8><title>TE console</title>\
         <body style='font:16px/1.4 system-ui;max-width:40rem;margin:3rem auto;padding:0 1rem'>\
         <h1>TE console</h1>\
         <p>API is up. Build the Vue app and restart:</p>\
         <pre>cd web\nnpm install\nnpm run build</pre>\
         <p>Or run the Vite proxy against this server:</p>\
         <pre>cd web\nnpm run dev</pre>\
         <p>Looked for <code>{}</code>.</p>",
        state.dist.display()
    ))
}

struct Opts {
    addr: SocketAddr,
}

impl Opts {
    fn parse() -> Self {
        let mut addr: SocketAddr = "127.0.0.1:8787".parse().unwrap();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bind" => {
                    addr = args
                        .next()
                        .expect("--bind needs host:port")
                        .parse()
                        .expect("invalid --bind");
                }
                "-h" | "--help" => {
                    eprintln!("te-console [--bind 127.0.0.1:8787]");
                    std::process::exit(0);
                }
                other => panic!("unknown argument: {other}"),
            }
        }
        Self { addr }
    }
}

fn dist_dir() -> PathBuf {
    let from_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../web/dist");
    if from_crate.join("index.html").is_file() {
        return from_crate;
    }
    PathBuf::from("web/dist")
}

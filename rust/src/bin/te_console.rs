//! Local HTTP console: Vue SPA + JSON simulation API.

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use tennessee_eastman::catalog::catalog;
use tennessee_eastman::simulate::{run, SimulationRequest};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Clone)]
struct AppState {
    dist: PathBuf,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let dist = dist_dir();
    let state = AppState { dist: dist.clone() };

    let mut router = Router::new()
        .route("/api/catalog", get(get_catalog))
        .route("/api/simulate", post(post_simulate));

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

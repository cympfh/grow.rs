use axum::{extract::Request, middleware::{self, Next}, response::Response, routing::get, Router};
use std::net::SocketAddr;

use crate::handler::{handle_path, handle_reload_events, handle_reload_js, handle_root, AppState};

async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    println!("{} {} {} {:?}", method, uri, status, duration);

    response
}

pub async fn start(state: AppState, host: &str, port: u16) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/__reload__", get(handle_reload_events))
        .route("/__reload__.js", get(handle_reload_js))
        .route("/*path", get(handle_path))
        .layer(middleware::from_fn(logging_middleware))
        .with_state(state);

    // 空いているポートを探す
    let mut current_port = port;
    let listener = loop {
        let addr: SocketAddr = format!("{}:{}", host, current_port)
            .parse()
            .expect("Invalid address");

        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Starting server on http://{}", addr);
                break listener;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                println!(
                    "Port {} is already in use, trying {}...",
                    current_port,
                    current_port + 1
                );
                current_port += 1;
                if current_port > port + 100 {
                    eprintln!("Could not find available port after 100 attempts");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    };

    axum::serve(listener, app).await
}

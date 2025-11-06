use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response, Sse},
    response::sse::{Event, KeepAlive},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::markdown;

#[derive(Clone)]
pub struct AppState {
    pub base_dir: Arc<PathBuf>,
    pub reload_tx: broadcast::Sender<()>,
}

pub async fn handle_root(State(state): State<AppState>) -> Response {
    handle_directory(&state.base_dir, "".to_string()).await
}

pub async fn handle_path(State(state): State<AppState>, Path(path): Path<String>) -> Response {
    let full_path = state.base_dir.join(&path);

    // セキュリティチェック: base_dir外へのアクセスを防ぐ
    if !full_path.starts_with(&*state.base_dir) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    if full_path.is_dir() {
        handle_directory(&full_path, path).await
    } else {
        handle_file(&full_path, &path).await
    }
}

async fn handle_directory(dir_path: &PathBuf, relative_path: String) -> Response {
    let mut entries = match tokio::fs::read_dir(dir_path).await {
        Ok(entries) => entries,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot read directory").into_response()
        }
    };

    let mut html = String::from("<html><head><title>Directory Listing</title>");
    html.push_str("<script src=\"/__reload__.js\"></script>");
    html.push_str("</head><body>");
    html.push_str(&format!("<h1>Directory: /{}</h1>", relative_path));
    html.push_str("<ul>");

    // 親ディレクトリへのリンク
    if !relative_path.is_empty() {
        let parent = if relative_path.contains('/') {
            relative_path.rsplitn(2, '/').nth(1).unwrap()
        } else {
            ""
        };
        html.push_str(&format!("<li><a href=\"/{}\">..</a></li>", parent));
    }

    let mut items = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(file_name) = entry.file_name().into_string() {
            let is_dir = entry.path().is_dir();
            items.push((file_name, is_dir));
        }
    }
    items.sort();

    for (name, is_dir) in items {
        let link_path = if relative_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", relative_path, name)
        };
        let display = if is_dir { format!("{}/", name) } else { name };
        html.push_str(&format!(
            "<li><a href=\"/{}\">{}</a></li>",
            link_path, display
        ));
    }

    html.push_str("</ul></body></html>");
    Html(html).into_response()
}

async fn handle_file(file_path: &PathBuf, _relative_path: &str) -> Response {
    let extension = file_path.extension().and_then(|s| s.to_str());

    // マークダウンファイルの場合はunidocで変換
    if matches!(extension, Some("md") | Some("mkd")) {
        match markdown::convert_to_html(file_path).await {
            Ok(html) => Html(html).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Markdown conversion failed: {}", e),
            )
                .into_response(),
        }
    } else {
        // その他のファイルはそのまま返す
        match tokio::fs::read(file_path).await {
            Ok(contents) => contents.into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cannot read file").into_response(),
        }
    }
}

pub async fn handle_reload_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.reload_tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|_| Ok(Event::default().data("reload")));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn handle_reload_js() -> Response {
    let js = include_str!("reload.js");
    (
        StatusCode::OK,
        [("Content-Type", "application/javascript")],
        js,
    )
        .into_response()
}

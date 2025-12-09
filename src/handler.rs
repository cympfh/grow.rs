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

    // パスを正規化してセキュリティチェック
    let canonical_path = match full_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // ファイルが存在しない場合はcanonicalize失敗するので、
            // 親ディレクトリで正規化してチェック
            let parent = full_path.parent();
            match parent {
                Some(p) => match p.canonicalize() {
                    Ok(canonical_parent) => {
                        // 親が base_dir 外ならFORBIDDEN
                        if !canonical_parent.starts_with(&*state.base_dir) {
                            return (StatusCode::FORBIDDEN, "Access denied").into_response();
                        }
                        // ファイルが存在しない
                        return handle_not_found(&path).await;
                    }
                    Err(_) => return handle_not_found(&path).await,
                },
                None => return handle_not_found(&path).await,
            }
        }
    };

    // セキュリティチェック: base_dir外へのアクセスを防ぐ
    if !canonical_path.starts_with(&*state.base_dir) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    if canonical_path.is_dir() {
        handle_directory(&canonical_path, path).await
    } else {
        handle_file(&canonical_path, &path, &state.base_dir).await
    }
}

async fn handle_not_found(path: &str) -> Response {
    // 親ディレクトリへのパスを計算
    let parent_path = if path.contains('/') {
        path.rsplitn(2, '/').nth(1).unwrap_or("")
    } else {
        ""
    };

    let html = format!(r#"<!DOCTYPE html>
<html><head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>404 Not Found</title>
<script src="/__reload__.js"></script>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: #333;
}}
.container {{
    max-width: 600px;
    width: 100%;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 20px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    padding: 3rem 2.5rem;
    text-align: center;
    animation: fadeIn 0.5s ease;
}}
@keyframes fadeIn {{
    from {{ opacity: 0; transform: translateY(20px); }}
    to {{ opacity: 1; transform: translateY(0); }}
}}
.error-code {{
    font-size: 8rem;
    font-weight: 900;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    line-height: 1;
    margin-bottom: 1rem;
    animation: pulse 2s ease-in-out infinite;
}}
@keyframes pulse {{
    0%, 100% {{ transform: scale(1); }}
    50% {{ transform: scale(1.05); }}
}}
h1 {{
    font-size: 2rem;
    font-weight: 700;
    margin-bottom: 1rem;
    color: #374151;
}}
.path {{
    font-size: 1rem;
    color: #6b7280;
    margin-bottom: 2rem;
    word-break: break-all;
    padding: 1rem;
    background: #f3f4f6;
    border-radius: 8px;
    font-family: 'Monaco', 'Courier New', monospace;
}}
.actions {{
    display: flex;
    flex-direction: column;
    gap: 1rem;
}}
a {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 1rem 2rem;
    text-decoration: none;
    border-radius: 12px;
    font-size: 1rem;
    font-weight: 600;
    transition: all 0.3s ease;
}}
.primary {{
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}}
.primary:hover {{
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
}}
.secondary {{
    background: #f3f4f6;
    color: #374151;
}}
.secondary:hover {{
    background: #e5e7eb;
    transform: translateY(-2px);
}}
.icon {{
    margin-right: 0.5rem;
    font-size: 1.25rem;
}}
@media (max-width: 640px) {{
    .container {{ padding: 2rem 1.5rem; }}
    .error-code {{ font-size: 6rem; }}
    h1 {{ font-size: 1.5rem; }}
}}
</style>
</head><body>
<div class="container">
    <div class="error-code">404</div>
    <h1>Page Not Found</h1>
    <div class="path">/{}</div>
    <div class="actions">
        <a href="/{}" class="primary">
            <span class="icon">⬆️</span>
            Go to Parent Directory
        </a>
        <a href="/" class="secondary">
            <span class="icon">🏠</span>
            Go to Home
        </a>
    </div>
</div>
</body></html>"#, path, parent_path);

    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

async fn handle_directory(dir_path: &PathBuf, relative_path: String) -> Response {
    let mut entries = match tokio::fs::read_dir(dir_path).await {
        Ok(entries) => entries,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot read directory").into_response()
        }
    };

    let mut html = String::from(r#"<!DOCTYPE html>
<html><head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Directory Listing</title>
<script src="/__reload__.js"></script>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    padding: 2rem;
    color: #333;
}
.container {
    max-width: 900px;
    margin: 0 auto;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 20px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    padding: 2.5rem;
    animation: fadeIn 0.5s ease;
}
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
}
h1 {
    font-size: 2rem;
    font-weight: 700;
    margin-bottom: 1.5rem;
    color: #667eea;
    border-bottom: 3px solid #667eea;
    padding-bottom: 0.75rem;
    word-break: break-all;
}
.path {
    font-size: 1rem;
    color: #666;
    font-weight: 400;
}
ul {
    list-style: none;
}
li {
    border-bottom: 1px solid #e5e7eb;
}
li:last-child {
    border-bottom: none;
}
a {
    display: flex;
    align-items: center;
    padding: 1rem 1.25rem;
    text-decoration: none;
    color: #374151;
    font-size: 1rem;
    transition: all 0.2s ease;
    border-radius: 8px;
}
a:hover {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    transform: translateX(8px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}
.icon {
    margin-right: 1rem;
    font-size: 1.5rem;
    min-width: 1.5rem;
}
.dir { color: #667eea; }
.file { color: #9ca3af; }
a.markdown { font-weight: 600; }
a:hover .icon { transform: scale(1.2); transition: transform 0.2s; }
.parent {
    font-weight: 600;
    color: #667eea;
}
.image-item {
    position: relative;
}
.image-item a {
    cursor: pointer;
}
.thumbnail {
    width: 80px;
    height: 80px;
    object-fit: cover;
    border-radius: 8px;
    margin-right: 1rem;
}
.modal {
    display: none;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.9);
    z-index: 10000;
    align-items: center;
    justify-content: center;
}
.modal.active {
    display: flex;
}
.modal-content {
    position: relative;
    max-width: 90%;
    max-height: 90%;
    display: flex;
    align-items: center;
    justify-content: center;
}
.modal-image {
    max-width: 100%;
    max-height: 90vh;
    object-fit: contain;
    border-radius: 8px;
}
.modal-close {
    position: absolute;
    top: 20px;
    right: 20px;
    color: white;
    font-size: 2rem;
    cursor: pointer;
    background: rgba(0, 0, 0, 0.5);
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s;
}
.modal-close:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: rotate(90deg);
}
.modal-link {
    position: absolute;
    top: 20px;
    right: 80px;
    background: rgba(0, 0, 0, 0.5);
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    text-decoration: none;
    font-size: 0.9rem;
    transition: all 0.3s;
}
.modal-link:hover {
    background: rgba(255, 255, 255, 0.2);
}
.modal-nav {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    background: rgba(0, 0, 0, 0.5);
    color: white;
    font-size: 2rem;
    cursor: pointer;
    width: 50px;
    height: 50px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s;
}
.modal-nav:hover {
    background: rgba(255, 255, 255, 0.2);
}
.modal-nav.prev {
    left: 20px;
}
.modal-nav.next {
    right: 20px;
}
@media (max-width: 640px) {
    body { padding: 1rem; }
    .container { padding: 1.5rem; }
    h1 { font-size: 1.5rem; }
    .thumbnail { width: 60px; height: 60px; }
}
</style>
</head><body><div class="container">"#);

    html.push_str(&format!(
        "<h1><span class=\"path\">📁 /{}</span></h1>",
        if relative_path.is_empty() { "Home".to_string() } else { relative_path.clone() }
    ));
    html.push_str("<ul>");

    // 親ディレクトリへのリンク
    if !relative_path.is_empty() {
        let parent = if relative_path.contains('/') {
            relative_path.rsplitn(2, '/').nth(1).unwrap()
        } else {
            ""
        };
        html.push_str(&format!(
            "<li><a href=\"/{}\" class=\"parent\"><span class=\"icon\">⬆️</span>Parent Directory</a></li>",
            parent
        ));
    }

    let mut items = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(file_name) = entry.file_name().into_string() {
            let is_dir = entry.path().is_dir();
            items.push((file_name, is_dir));
        }
    }
    items.sort();

    let mut image_paths = Vec::new();
    for (name, is_dir) in items {
        let link_path = if relative_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", relative_path, name)
        };

        // 画像ファイルの判定
        let is_image = !is_dir && (
            name.ends_with(".jpg") || name.ends_with(".jpeg") ||
            name.ends_with(".png") || name.ends_with(".gif") ||
            name.ends_with(".webp") || name.ends_with(".svg") ||
            name.ends_with(".JPG") || name.ends_with(".JPEG") ||
            name.ends_with(".PNG") || name.ends_with(".GIF") ||
            name.ends_with(".WEBP") || name.ends_with(".SVG")
        );

        if is_image {
            image_paths.push(link_path.clone());
        }

        // Markdownファイルには特別なアイコンを使用
        let (icon, icon_class, link_class) = if is_dir {
            ("📁", "dir", "")
        } else if name.ends_with(".md") || name.ends_with(".mkd") {
            ("📝", "file", " markdown")
        } else if is_image {
            ("🖼️", "file", "")
        } else {
            ("📄", "file", "")
        };

        if is_image {
            html.push_str(&format!(
                "<li class=\"image-item\"><a href=\"javascript:void(0)\" onclick=\"openModal('{}', {})\" class=\"{}\"><img src=\"/{}\" class=\"thumbnail\" alt=\"{}\" loading=\"lazy\"><span>{}</span></a></li>",
                link_path, image_paths.len() - 1, link_class.trim(), link_path, name, name
            ));
        } else {
            html.push_str(&format!(
                "<li><a href=\"/{}\" class=\"{}\"><span class=\"icon {}\">{}</span>{}</a></li>",
                link_path, link_class.trim(), icon_class, icon, name
            ));
        }
    }

    html.push_str("</ul></div>");

    // モーダルの追加
    html.push_str(r#"<div id="imageModal" class="modal">
    <div class="modal-content">
        <a id="modalLink" class="modal-link" href="" target="_blank">元ファイル</a>
        <div class="modal-close" onclick="closeModal()">×</div>
        <div class="modal-nav prev" onclick="prevImage()">‹</div>
        <div class="modal-nav next" onclick="nextImage()">›</div>
        <img id="modalImage" class="modal-image" src="" alt="">
    </div>
</div>
<script>
const imagePaths = "#);
    html.push_str(&serde_json::to_string(&image_paths).unwrap_or_else(|_| "[]".to_string()));
    html.push_str(r#";
let currentImageIndex = 0;

function openModal(imagePath, index) {
    currentImageIndex = index;
    updateModalImage();
    document.getElementById('imageModal').classList.add('active');
}

function closeModal() {
    document.getElementById('imageModal').classList.remove('active');
}

function updateModalImage() {
    const imagePath = imagePaths[currentImageIndex];
    document.getElementById('modalImage').src = '/' + imagePath;
    document.getElementById('modalLink').href = '/' + imagePath;
}

function nextImage() {
    currentImageIndex = (currentImageIndex + 1) % imagePaths.length;
    updateModalImage();
}

function prevImage() {
    currentImageIndex = (currentImageIndex - 1 + imagePaths.length) % imagePaths.length;
    updateModalImage();
}

document.getElementById('imageModal').addEventListener('click', function(e) {
    if (e.target === this) {
        closeModal();
    }
});

document.addEventListener('keydown', function(e) {
    const modal = document.getElementById('imageModal');
    if (modal.classList.contains('active')) {
        if (e.key === 'ArrowLeft') {
            prevImage();
        } else if (e.key === 'ArrowRight') {
            nextImage();
        } else if (e.key === 'Escape') {
            closeModal();
        }
    }
});
</script>
</body></html>"#);
    Html(html).into_response()
}

async fn handle_file(file_path: &PathBuf, relative_path: &str, base_dir: &PathBuf) -> Response {
    let extension = file_path.extension().and_then(|s| s.to_str());

    // マークダウンファイルの場合はunidocで変換
    if matches!(extension, Some("md") | Some("mkd")) {
        match markdown::convert_to_html(file_path, relative_path, base_dir).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_state(base_dir: PathBuf) -> AppState {
        let (reload_tx, _) = broadcast::channel(100);
        AppState {
            base_dir: Arc::new(base_dir),
            reload_tx,
        }
    }

    #[tokio::test]
    async fn test_path_traversal_protection() {
        let temp_dir = std::env::temp_dir().join("mvu_test_traversal");
        fs::create_dir_all(&temp_dir).unwrap();

        // temp_dirの親ディレクトリに実際にファイルを作成
        let parent_dir = temp_dir.parent().unwrap();
        let target_file = parent_dir.join("secret.txt");
        fs::write(&target_file, "secret data").unwrap();

        let state = create_test_state(temp_dir.clone());

        // パストラバーサル攻撃の試み: 親ディレクトリのファイルにアクセス
        let response = handle_path(State(state), Path("../secret.txt".to_string())).await;
        let status = response.status();

        // base_dir外のアクセスは403 FORBIDDENになるべき
        assert_eq!(status, StatusCode::FORBIDDEN);

        // クリーンアップ
        fs::remove_file(&target_file).ok();
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_nonexistent_path() {
        let temp_dir = std::env::temp_dir().join("mvu_test_nonexist");
        fs::create_dir_all(&temp_dir).unwrap();

        let state = create_test_state(temp_dir.clone());

        let response = handle_path(State(state), Path("nonexistent.txt".to_string())).await;
        let status = response.status();

        assert_eq!(status, StatusCode::NOT_FOUND);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_directory_listing() {
        let temp_dir = std::env::temp_dir().join("mvu_test_dir");
        fs::create_dir_all(&temp_dir).unwrap();

        // テスト用のファイルを作成
        fs::write(temp_dir.join("test.txt"), "test content").unwrap();
        fs::write(temp_dir.join("test.md"), "# Test").unwrap();

        let response = handle_directory(&temp_dir, "".to_string()).await;
        let status = response.status();

        assert_eq!(status, StatusCode::OK);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_markdown_file_detection() {
        let temp_dir = std::env::temp_dir().join("mvu_test_md");
        fs::create_dir_all(&temp_dir).unwrap();

        // Markdownファイルを作成
        fs::write(temp_dir.join("test.md"), "# Hello World").unwrap();
        fs::write(temp_dir.join("test.mkd"), "# Hello MKD").unwrap();

        // .md ファイルのテスト
        let md_path = temp_dir.join("test.md");
        let extension = md_path.extension().and_then(|s| s.to_str());
        assert!(matches!(extension, Some("md")));

        // .mkd ファイルのテスト
        let mkd_path = temp_dir.join("test.mkd");
        let extension = mkd_path.extension().and_then(|s| s.to_str());
        assert!(matches!(extension, Some("mkd")));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_reload_js_content() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let response = handle_reload_js().await;
            assert_eq!(response.status(), StatusCode::OK);
        });
    }

    #[tokio::test]
    async fn test_image_file_detection() {
        let temp_dir = std::env::temp_dir().join("mvu_test_images");
        fs::create_dir_all(&temp_dir).unwrap();

        // 画像ファイルを作成
        fs::write(temp_dir.join("test.jpg"), "fake image").unwrap();
        fs::write(temp_dir.join("test.png"), "fake image").unwrap();
        fs::write(temp_dir.join("test.gif"), "fake image").unwrap();
        fs::write(temp_dir.join("test.webp"), "fake image").unwrap();
        fs::write(temp_dir.join("test.svg"), "fake image").unwrap();
        fs::write(temp_dir.join("document.txt"), "text file").unwrap();

        let response = handle_directory(&temp_dir, "".to_string()).await;
        let status = response.status();
        assert_eq!(status, StatusCode::OK);

        // レスポンスボディを取得
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // 画像ファイルがサムネイル表示されているか確認
        assert!(body_str.contains("test.jpg"));
        assert!(body_str.contains("test.png"));
        assert!(body_str.contains("class=\"thumbnail\""));
        assert!(body_str.contains("openModal"));
        assert!(body_str.contains("imageModal"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}

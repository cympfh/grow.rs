use std::path::PathBuf;
use tokio::process::Command;

const RELOAD_HTML: &str = r#"<script src="/__reload__.js"></script>"#;

pub async fn convert_to_html(file_path: &PathBuf) -> Result<String, String> {
    // 一時ファイルにreload.htmlを書き出し
    let temp_html = std::env::temp_dir().join("grow_reload.html");
    std::fs::write(&temp_html, RELOAD_HTML)
        .map_err(|e| format!("Failed to write reload.html: {}", e))?;

    // コマンドをログ出力
    eprintln!(
        "[unidoc] Running: unidoc -s -H {} {}",
        temp_html.display(),
        file_path.display()
    );

    let output = Command::new("unidoc")
        .arg("-s")
        .arg("-H")
        .arg(&temp_html)
        .arg("--")
        .arg(file_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute unidoc: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 in output: {}", e))
    } else {
        Err(format!(
            "unidoc failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

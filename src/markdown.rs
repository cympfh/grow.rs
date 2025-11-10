use std::path::PathBuf;
use tokio::process::Command;
use std::fs;
use std::io::{BufRead, BufReader};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

const RELOAD_HTML: &str = r#"<script src="/__reload__.js"></script>
<style>
* { box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
    background: #ffffff;
    min-height: 100vh;
    padding: 2rem 1rem 2rem 1rem;
    line-height: 1.7;
    color: #333;
    max-width: 850px;
    margin: 0 auto;
}
#side-menu {
    position: fixed;
    left: -300px;
    top: 0;
    width: 300px;
    height: 100vh;
    background: #f9fafb;
    box-shadow: 2px 0 10px rgba(0,0,0,0.1);
    transition: left 0.3s ease;
    z-index: 1000;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}
#side-menu.open {
    left: 0;
}
#menu-toggle {
    position: fixed;
    left: 1rem;
    bottom: 1rem;
    width: 40px;
    height: 40px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
    z-index: 1001;
    transition: all 0.3s ease;
}
#menu-toggle:hover {
    transform: scale(1.1);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.5);
}
.menu-tabs {
    display: flex;
    background: #fff;
    border-bottom: 2px solid #e5e7eb;
}
.menu-tab {
    flex: 1;
    padding: 1rem;
    text-align: center;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    color: #666;
    transition: all 0.2s;
    border-bottom: 3px solid transparent;
}
.menu-tab:hover {
    background: #f3f4f6;
    color: #667eea;
}
.menu-tab.active {
    color: #667eea;
    font-weight: 600;
    border-bottom-color: #667eea;
}
.menu-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
}
.menu-panel {
    display: none;
}
.menu-panel.active {
    display: block;
}
#toc-panel ul {
    list-style: none;
    padding-left: 0;
}
#toc-panel li {
    margin: 0.5rem 0;
}
#toc-panel a {
    color: #374151;
    text-decoration: none;
    display: block;
    padding: 0.5rem;
    border-radius: 6px;
    transition: all 0.2s;
}
#toc-panel a:hover {
    background: #667eea;
    color: white;
    transform: translateX(4px);
}
#toc-panel .toc-h1 { font-weight: 600; font-size: 1rem; }
#toc-panel .toc-h2 { padding-left: 1rem; font-size: 0.95rem; }
#toc-panel .toc-h3 { padding-left: 2rem; font-size: 0.9rem; }
#toc-panel .toc-h4 { padding-left: 3rem; font-size: 0.85rem; }
#file-tree ul {
    list-style: none;
    padding-left: 0;
}
#file-tree li {
    margin: 0.25rem 0;
}
#file-tree a {
    color: #374151;
    text-decoration: none;
    display: block;
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    transition: all 0.2s;
}
#file-tree a:hover {
    background: #667eea;
    color: white;
}
#file-tree .dir {
    font-weight: 600;
    color: #667eea;
}
#file-tree .dir::before {
    content: '📁 ';
}
#file-tree .file::before {
    content: '📄 ';
}
#file-tree .markdown {
    font-weight: 600;
}
#file-tree .markdown::before {
    content: '📝 ';
}
#file-tree .nested {
    padding-left: 1rem;
}
@media (max-width: 768px) {
    #side-menu {
        width: 100%;
        left: -100%;
    }
    #side-menu.open {
        left: 0;
    }
}
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
}
h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 700;
    line-height: 1.3;
    color: #667eea;
}
h1 {
    font-size: 2.5rem;
    border-bottom: 3px solid #667eea;
    padding-bottom: 0.5rem;
}
h2 {
    font-size: 2rem;
    border-bottom: 2px solid #e5e7eb;
    padding-bottom: 0.4rem;
}
h3 { font-size: 1.5rem; }
h4 { font-size: 1.25rem; }
p { margin: 1rem 0; }
a {
    color: #667eea;
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: all 0.2s ease;
}
a:hover {
    border-bottom-color: #667eea;
    color: #764ba2;
}
code {
    background: #f3f4f6;
    padding: 0.2em 0.4em;
    border-radius: 6px;
    font-family: 'Fira Code', 'Consolas', 'Monaco', monospace;
    font-size: 0.9em;
    color: #e83e8c;
}
pre {
    background: #282c34;
    color: #abb2bf;
    padding: 1.5rem;
    border-radius: 12px;
    overflow-x: auto;
    margin: 1.5rem 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
pre code {
    background: none;
    color: inherit;
    padding: 0;
    font-size: 0.95em;
}
blockquote {
    border-left: 4px solid #667eea;
    padding-left: 1.5rem;
    margin: 1.5rem 0;
    color: #555;
    background: #f9fafb;
    padding: 1rem 1.5rem;
    border-radius: 0 8px 8px 0;
}
ul, ol {
    padding-left: 2rem;
    margin: 1rem 0;
}
li { margin: 0.5rem 0; }
table {
    border-collapse: collapse;
    width: 100%;
    margin: 1.5rem 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    overflow: hidden;
}
th, td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid #e5e7eb;
}
th {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    font-weight: 600;
}
tr:hover {
    background: #f9fafb;
}
img {
    max-width: 100%;
    height: auto;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    margin: 1rem 0;
}
hr {
    border: none;
    border-top: 2px solid #e5e7eb;
    margin: 2rem 0;
}
@media (max-width: 768px) {
    body { padding: 1rem 0.5rem; }
    .markdown-body { padding: 1.5rem; }
    h1 { font-size: 2rem; }
    h2 { font-size: 1.5rem; }
}
</style>"#;

pub async fn convert_to_html(file_path: &PathBuf, relative_path: &str, base_dir: &PathBuf) -> Result<String, String> {
    // ヘッダー用の一時ファイル
    let temp_header = std::env::temp_dir().join("mvu_reload_header.html");
    std::fs::write(&temp_header, RELOAD_HTML)
        .map_err(|e| format!("Failed to write header.html: {}", e))?;

    // サイドメニュー用の一時ファイル
    let side_menu_html = generate_side_menu(file_path, relative_path, base_dir)?;
    let temp_body = std::env::temp_dir().join("mvu_side_menu.html");
    std::fs::write(&temp_body, side_menu_html)
        .map_err(|e| format!("Failed to write side_menu.html: {}", e))?;

    // コマンドをログ出力
    eprintln!("[unidoc] Running: unidoc for {}", file_path.display());

    let output = Command::new("unidoc")
        .arg("-s")
        .arg("-H")
        .arg(&temp_header)
        .arg("-B")
        .arg(&temp_body)
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

fn generate_side_menu(file_path: &PathBuf, relative_path: &str, base_dir: &PathBuf) -> Result<String, String> {
    let toc = extract_toc_from_markdown(file_path)?;
    let file_tree = generate_file_tree(base_dir, relative_path)?;

    let menu_html = format!(r#"
<button id="menu-toggle" aria-label="Toggle menu">☰</button>
<div id="side-menu">
    <div class="menu-tabs">
        <button class="menu-tab active" data-tab="toc">📑 目次</button>
        <button class="menu-tab" data-tab="files">📁 ファイル</button>
    </div>
    <div class="menu-content">
        <div id="toc-panel" class="menu-panel active">
            {}
        </div>
        <div id="file-tree" class="menu-panel">
            {}
        </div>
    </div>
</div>
<script>
(function() {{
    const menuToggle = document.getElementById('menu-toggle');
    const sideMenu = document.getElementById('side-menu');
    const menuTabs = document.querySelectorAll('.menu-tab');
    const menuPanels = document.querySelectorAll('.menu-panel');

    menuToggle.addEventListener('click', function() {{
        sideMenu.classList.toggle('open');
        document.body.classList.toggle('menu-open');
    }});

    menuTabs.forEach(tab => {{
        tab.addEventListener('click', function() {{
            const tabName = this.getAttribute('data-tab');

            menuTabs.forEach(t => t.classList.remove('active'));
            this.classList.add('active');

            menuPanels.forEach(panel => {{
                panel.classList.remove('active');
            }});

            if (tabName === 'toc') {{
                document.getElementById('toc-panel').classList.add('active');
            }} else if (tabName === 'files') {{
                document.getElementById('file-tree').classList.add('active');
            }}
        }});
    }});

    // TOC links smooth scroll
    document.querySelectorAll('#toc-panel a').forEach(link => {{
        link.addEventListener('click', function(e) {{
            e.preventDefault();
            const targetId = this.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            if (targetElement) {{
                targetElement.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
            }}
        }});
    }});

    // Close menu when clicking outside
    document.addEventListener('click', function(e) {{
        const isMenuOpen = sideMenu.classList.contains('open');
        const clickedInsideMenu = sideMenu.contains(e.target);
        const clickedMenuToggle = menuToggle.contains(e.target);

        if (isMenuOpen && !clickedInsideMenu && !clickedMenuToggle) {{
            sideMenu.classList.remove('open');
            document.body.classList.remove('menu-open');
        }}
    }});
}})();
</script>
"#, toc, file_tree);

    Ok(menu_html)
}

fn extract_toc_from_markdown(file_path: &PathBuf) -> Result<String, String> {
    let file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open markdown file: {}", e))?;

    let reader = BufReader::new(file);
    let mut toc = String::from("<ul>");
    let mut counter = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed = line.trim();

        if trimmed.starts_with('#') && !trimmed.starts_with("####") {
            let mut level = 0;
            let mut chars = trimmed.chars();

            // Count # characters
            while let Some('#') = chars.next() {
                level += 1;
                if level > 4 {
                    break;
                }
            }

            if level > 0 && level <= 4 {
                // Extract text after #
                let text = trimmed[level..].trim().to_string();

                if !text.is_empty() {
                    // Generate ID using the same logic as unidoc
                    let id = format!("{}-{}", level, percent_encode(&text));
                    counter += 1;

                    toc.push_str(&format!(
                        "<li class=\"toc-h{}\"><a href=\"#{}\">{}</a></li>",
                        level, id, html_escape(&text)
                    ));
                }
            }
        }
    }

    toc.push_str("</ul>");

    if counter == 0 {
        return Ok(String::from("<p>目次がありません</p>"));
    }

    Ok(toc)
}

fn percent_encode(input: &str) -> String {
    const CUSTOM_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_');
    utf8_percent_encode(input, CUSTOM_ENCODE_SET).to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn generate_file_tree(base_dir: &PathBuf, current_path: &str) -> Result<String, String> {
    fn build_tree(dir: &PathBuf, prefix: &str, current: &str, depth: usize) -> Result<String, String> {
        if depth > 3 {
            return Ok(String::new());
        }

        let mut html = String::from("<ul class=\"nested\">");

        let mut entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by_key(|e| e.path());

        // ディレクトリを先に
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            b_is_dir.cmp(&a_is_dir)
        });

        for entry in entries {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if name.starts_with('.') {
                continue;
            }

            let link_path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };

            let is_current = link_path == current;
            let style = if is_current { " style=\"background: #667eea; color: white;\"" } else { "" };

            if path.is_dir() {
                html.push_str(&format!(
                    r#"<li><a href="/{}" class="dir"{}>{}</a>"#,
                    html_escape(&link_path), style, html_escape(name)
                ));

                if current.starts_with(&link_path) {
                    if let Ok(subtree) = build_tree(&path, &link_path, current, depth + 1) {
                        html.push_str(&subtree);
                    }
                }

                html.push_str("</li>");
            } else {
                let class = if name.ends_with(".md") || name.ends_with(".mkd") {
                    "markdown"
                } else {
                    "file"
                };

                html.push_str(&format!(
                    r#"<li><a href="/{}" class="{}"{}>{}</a></li>"#,
                    html_escape(&link_path), class, style, html_escape(name)
                ));
            }
        }

        html.push_str("</ul>");
        Ok(html)
    }

    build_tree(base_dir, "", current_path, 0)
}

mod cli;
mod handler;
mod markdown;
mod server;
mod watcher;

use clap::Parser;
use std::sync::Arc;
use tokio::sync::broadcast;

use cli::Args;
use handler::AppState;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let base_dir = args.directory.canonicalize().expect("Invalid directory");

    // ファイル変更通知用のチャンネル
    let (reload_tx, _) = broadcast::channel(100);

    // ファイル監視を開始
    let watcher_tx = reload_tx.clone();
    let watch_path = base_dir.clone();
    std::thread::spawn(move || {
        if let Ok((_watcher, mut rx)) = watcher::start_watcher(watch_path) {
            while let Ok(()) = rx.blocking_recv() {
                let _ = watcher_tx.send(());
            }
        }
    });

    let state = AppState {
        base_dir: Arc::new(base_dir),
        reload_tx,
    };

    if let Err(e) = server::start(state, &args.host, args.port).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

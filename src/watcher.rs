use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::broadcast;

pub fn start_watcher(
    watch_path: PathBuf,
) -> Result<(RecommendedWatcher, broadcast::Receiver<()>), Box<dyn std::error::Error>> {
    let (tx, rx) = broadcast::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // ファイルの変更イベントを検知
                if matches!(
                    event.kind,
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                ) {
                    let _ = tx.send(());
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    Ok((watcher, rx))
}

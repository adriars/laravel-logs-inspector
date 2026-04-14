use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::app::AppEvent;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};

pub struct LogFileWatcher {
    path: PathBuf,
    file_tx: Sender<AppEvent>,
}

impl LogFileWatcher {
    pub fn new(path: PathBuf, file_tx: Sender<AppEvent>) -> LogFileWatcher {
        LogFileWatcher { path, file_tx }
    }

    pub fn start(self) {
        thread::spawn(move || {
            let (notify_tx, notify_rx) = mpsc::channel();
            let mut watcher = RecommendedWatcher::new(notify_tx, Config::default()).unwrap();
            watcher
                .watch(self.path.as_ref(), RecursiveMode::Recursive)
                .unwrap();

            for res in notify_rx {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {

                                if path.extension().map_or(false, |ext| ext == "log") {
                                    let _ = self.file_tx.send(AppEvent::FileCreated(path));
                                }
                            }
                        }
                        EventKind::Modify(ModifyKind::Data(_)) => {
                            for path in event.paths {

                                if path.extension().map_or(false, |ext| ext == "log") {
                                    // Checking metadata to ensure file isn't empty
                                    if path.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
                                        let _ = self.file_tx.send(AppEvent::FileUpdated(path));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

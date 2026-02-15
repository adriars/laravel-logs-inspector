use std::{
    sync::mpsc::{self, Sender},
    thread,
};

use crate::app::AppEvent;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::{DataChange, ModifyKind}};

pub struct LogFileWatcher {
    path: String,
    file_tx: Sender<AppEvent>,
}

impl LogFileWatcher {
    pub fn new(path: String, file_tx: Sender<AppEvent>) -> LogFileWatcher {
        LogFileWatcher {
            path: path,
            file_tx: file_tx,
        }
    }

    pub fn start(self) {
        thread::spawn(move || {
            let (notify_tx, notify_rx) = mpsc::channel();
            let mut watcher = RecommendedWatcher::new(notify_tx, Config::default()).unwrap();
            watcher
                .watch(self.path.as_ref(), RecursiveMode::NonRecursive)
                .unwrap();

            for res in notify_rx {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {
                                if path.extension().map_or(false, |ext| ext == "log") {
                                    let name =
                                        path.file_name().unwrap().to_string_lossy().into_owned();
                                    let _ = self.file_tx.send(AppEvent::FileCreated(name));
                                }
                            }
                        }

                        EventKind::Modify(ModifyKind::Data(_)) => {
                            for path in event.paths {
                                if path.extension().map_or(false, |ext| ext == "log") {
                                    let name =
                                        path.file_name().unwrap().to_string_lossy().into_owned();
                                    let _ = self.file_tx.send(AppEvent::FileUpdated(name));
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

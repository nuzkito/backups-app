use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use eframe::egui;

use crate::engine::{self, Ignore, Outcome, Progress};

enum Message {
    Progress(Progress),
    Finished(Outcome),
}

pub struct Task {
    rx: Receiver<Message>,
    cancel: Arc<AtomicBool>,
    /// Last progress received. `None` while the engine scans the source folders.
    pub progress: Option<Progress>,
}

impl Task {
    /// Starts the backup in a new thread. Requests a repaint of `ctx` for each message.
    pub fn spawn(
        ctx: &egui::Context,
        sources: Vec<PathBuf>,
        dest: PathBuf,
        ignore: Ignore,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let thread_cancel = cancel.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let send = |message| {
                let _ = tx.send(message);
                ctx.request_repaint();
            };
            let outcome = engine::run(&sources, &dest, &ignore, &thread_cancel, |p| {
                send(Message::Progress(p));
            });
            send(Message::Finished(outcome));
        });
        Self {
            rx,
            cancel,
            progress: None,
        }
    }

    /// Task with no thread behind it, which stays in progress (screenshots).
    #[cfg(test)]
    pub fn idle(progress: Option<Progress>) -> Self {
        let (tx, rx) = mpsc::channel();
        // Keeps the sender alive so that `poll` does not see a closed channel.
        std::mem::forget(tx);
        Self {
            rx,
            cancel: Arc::new(AtomicBool::new(false)),
            progress,
        }
    }

    /// Asks the engine to stop. The task finishes with `Outcome::Cancelled`.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Reads the pending messages. Returns the outcome when the backup finishes.
    pub fn poll(&mut self) -> Option<Outcome> {
        loop {
            match self.rx.try_recv() {
                Ok(Message::Progress(p)) => self.progress = Some(p),
                Ok(Message::Finished(outcome)) => return Some(outcome),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    return Some(Outcome::Failed(
                        "The backup process stopped unexpectedly.".into(),
                    ));
                }
            }
        }
    }
}

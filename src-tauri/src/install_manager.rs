use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::install::{self, InstallProgressEvent, ProgressCallback};
use crate::pipeline::DownloadProgressCallback;

// ── EventDispatcher trait ───────────────────────────────────

/// Abstracts Tauri event emission so `InstallManager` is testable
/// without a real `AppHandle`. Each method maps 1:1 to a Tauri event.
pub trait EventDispatcher: Send + Sync + 'static {
    fn emit_progress(&self, event: InstallProgressEvent);
    fn emit_complete(&self, result: install::InstallResult);
    fn emit_error(&self, error: String);
}

// ── InstallManager ───────────────────────────────────────────

/// Owns the install lifecycle: token management, progress callbacks,
/// and `tokio::spawn`. Replaces the old `InstallRegistry` in `lib.rs`.
///
/// The UI never runs concurrent installs, so at most one token is kept.
/// A new `start()` cancels any previous install.
pub struct InstallManager {
    token: Mutex<Option<CancellationToken>>,
}

impl InstallManager {
    pub fn new() -> Self {
        Self {
            token: Mutex::new(None),
        }
    }

    /// Start a cancellable install in a background task.
    ///
    /// Progress events are emitted through `dispatcher` during the
    /// download → extract → copy phases. On completion, either
    /// `emit_complete` or `emit_error` is called.
    ///
    /// Returns immediately with the install id (always `"current"` —
    /// we only support one at a time).
    pub fn start(
        self: &Arc<Self>,
        dispatcher: Arc<dyn EventDispatcher>,
        options: install::InstallOptions,
    ) -> Result<String, String> {
        let token = CancellationToken::new();
        {
            let mut slot = self
                .token
                .lock()
                .map_err(|_| "Internal error: state lock is poisoned".to_string())?;
            if let Some(old) = slot.take() {
                old.cancel();
            }
            *slot = Some(token.clone());
        }

        // Build progress callbacks from the dispatcher trait object.
        let progress_dispatcher = dispatcher.clone();
        let progress: ProgressCallback = Arc::new(move |event: InstallProgressEvent| {
            progress_dispatcher.emit_progress(event);
        });

        let download_dispatcher = dispatcher.clone();
        let download_progress: DownloadProgressCallback = Arc::new(move |bytes, total| {
            download_dispatcher.emit_progress(InstallProgressEvent {
                step: "download".to_string(),
                details: String::new(),
                current_bytes: Some(bytes),
                total_bytes: total,
            });
        });

        // Clone self (Arc) for the background task so it can clear
        // the token on completion without a separate shared slot.
        let manager_for_task = self.clone();
        let task_dispatcher = dispatcher.clone();

        tokio::spawn(async move {
            let res =
                install::install_minui_with_cancel(&options, progress, download_progress, token)
                    .await;

            // Clear the token so cancel_install is a no-op after completion.
            if let Ok(mut slot) = manager_for_task.token.lock() {
                *slot = None;
            }

            match res {
                Ok(r) => task_dispatcher.emit_complete(r),
                Err(e) => task_dispatcher.emit_error(e),
            }
        });

        Ok("current".to_string())
    }

    /// Cancel the in-flight install, if any. No-op when idle.
    pub fn cancel(&self) -> Result<(), String> {
        let slot = self
            .token
            .lock()
            .map_err(|_| "Internal error: state lock is poisoned".to_string())?;
        if let Some(token) = slot.as_ref() {
            token.cancel();
        }
        Ok(())
    }
}

impl Default for InstallManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── MockDispatcher (for tests) ───────────────────────────────

#[cfg(test)]
pub struct MockDispatcher {
    pub progress_events: Mutex<Vec<InstallProgressEvent>>,
    pub complete_results: Mutex<Vec<install::InstallResult>>,
    pub error_messages: Mutex<Vec<String>>,
}

#[cfg(test)]
impl MockDispatcher {
    pub fn new() -> Self {
        Self {
            progress_events: Mutex::new(Vec::new()),
            complete_results: Mutex::new(Vec::new()),
            error_messages: Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
impl EventDispatcher for MockDispatcher {
    fn emit_progress(&self, event: InstallProgressEvent) {
        self.progress_events.lock().unwrap().push(event);
    }

    fn emit_complete(&self, result: install::InstallResult) {
        self.complete_results.lock().unwrap().push(result);
    }

    fn emit_error(&self, error: String) {
        self.error_messages.lock().unwrap().push(error);
    }
}

#[cfg(test)]
#[path = "install_manager_tests.rs"]
mod tests;

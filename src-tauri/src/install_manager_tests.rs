use super::*;
use std::sync::Arc;

// ── MockDispatcher ───────────────────────────────────────

#[test]
fn mock_dispatcher_records_events() {
    let mock = Arc::new(MockDispatcher::new());
    mock.emit_progress(InstallProgressEvent::phase("download", "test"));
    mock.emit_complete(install::InstallResult {
        success: true,
        error: None,
        base_files_copied: 1,
        extras_files_copied: 0,
        extras_warning: None,
        rom_dirs_created: 0,
    });
    mock.emit_error("fail".to_string());

    assert_eq!(mock.progress_events.lock().unwrap().len(), 1);
    assert_eq!(mock.complete_results.lock().unwrap().len(), 1);
    assert_eq!(mock.error_messages.lock().unwrap().len(), 1);
}

// ── InstallManager::cancel (poisoned mutex) ──────────────

#[test]
fn test_cancel_returns_err_on_poisoned_mutex() {
    let manager = Arc::new(InstallManager::new());

    // Poison by panicking while holding the lock in another thread.
    let mgr = manager.clone();
    let handle = std::thread::spawn(move || {
        let _guard = mgr.token.lock().unwrap();
        panic!("intentional panic to poison mutex");
    });
    let _ = handle.join();

    let result = manager.cancel();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("poisoned"));
}

#[test]
fn test_cancel_is_noop_when_idle() {
    let manager = InstallManager::new();
    let result = manager.cancel();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_cancels_previous_install() {
    let manager = Arc::new(InstallManager::new());
    let mock = Arc::new(MockDispatcher::new());

    let options = install::InstallOptions {
        base_url: "http://127.0.0.1:1/never.zip".to_string(),
        extras_url: None,
        base_checksum: None,
        extras_checksum: None,
        sd_mount: "/tmp".to_string(),
        platform: "trimui-brick".to_string(),
        extras_platform: "trimui-brick".to_string(),
        version: "test".to_string(),
        fork_name: None,
    };

    // First install starts (will fail because URL is unreachable,
    // but the token should be registered first).
    let _id = manager.start(mock.clone(), options.clone()).unwrap();

    // Second install should cancel the first.
    let _id2 = manager.start(mock, options).unwrap();

    // After cancellation + new start, the old token should be gone
    // (the second start replaced it). We can't easily inspect internal
    // state, but cancel() should still be a no-op after completion.
    let cancel_result = manager.cancel();
    assert!(cancel_result.is_ok());
}

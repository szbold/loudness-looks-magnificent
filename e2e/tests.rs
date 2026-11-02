//! E2E (CONVENTIONS.md): drive the real binary headless — no audio device, no
//! window, no GPU (FR-M1-6). These tests are the CI smoke gate.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)

use std::process::Command;

/// The binary this suite exercises, wired by cargo.
const BIN: &str = env!("CARGO_BIN_EXE_loudness-looks-magnificent");

#[test]
fn smoke_renders_requested_frames_and_reports() {
    let out = Command::new(BIN)
        .args([
            "--headless",
            "--fake-audio",
            "7",
            "--frames",
            "180",
            "--fps-report",
        ])
        .output()
        .expect("binary should run");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"frames\":180"), "stdout: {stdout}");
    assert!(stdout.contains("\"p99_ms\""), "stdout: {stdout}");
}

#[test]
fn zero_frames_is_a_usage_error() {
    let out = Command::new(BIN)
        .args(["--headless", "--fake-audio", "7", "--frames", "0"])
        .output()
        .expect("binary should run");
    assert!(!out.status.success(), "zero frames must not succeed");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--frames"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn window_mode_reports_milestone_honestly() {
    let out = Command::new(BIN).output().expect("binary should run");
    assert!(!out.status.success(), "window mode is not implemented yet");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("FR-M1-1"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

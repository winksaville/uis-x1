//! Smoke test: the binary runs, shows the text, exits cleanly, and takes its fixed time.
//!
//! Panicking on a setup failure is the right test behavior, so the crate's `expect_used` lint
//! is allowed here as it is in unit tests.
#![allow(clippy::expect_used)]

use std::process::Command;
use std::time::Instant;

use uis_x1::{DISPLAY_TIME, TEXT};

/// Run the binary with stdout captured and check the text, the exit status, and the elapsed time.
///
/// Cargo sets `CARGO_BIN_EXE_<name>` only for integration tests, so this cannot live in a unit
/// test module inside the binary.
#[test]
fn shows_hello_world_for_the_fixed_time() {
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_uis-x1"))
        .output()
        .expect("binary runs");
    let elapsed = start.elapsed();
    assert!(output.status.success(), "exit status {}", output.status);
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains(TEXT), "shows the text");
    assert!(elapsed >= DISPLAY_TIME, "ran for {elapsed:?}");
}

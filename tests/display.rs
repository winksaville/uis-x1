//! Smoke test: the binary shows the text, waits for Return, and exits cleanly once it arrives.
//!
//! Panicking on a setup failure is the right test behavior, so the crate's `expect_used` lint
//! is allowed here as it is in unit tests.
#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use uis_x1::TEXT;

/// How long the binary runs before the test checks that it is still waiting.
const PAUSE: Duration = Duration::from_millis(200);

/// How long the binary may take to exit once it has its newline.
const DEADLINE: Duration = Duration::from_secs(5);

/// Run the binary with stdin and stdout piped, check that it waits, send a newline, and check
/// the exit and the text.
///
/// - Cargo sets `CARGO_BIN_EXE_<name>` only for integration tests, so this cannot live in a
///   unit test module inside the binary.
/// - Stdin stays open until the exit, since closing it would end the display through end of
///   input and hide a broken Return.
#[test]
fn shows_hello_world_until_return() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_uis-x1"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("binary starts");
    let mut stdin = child.stdin.take().expect("stdin is piped");

    thread::sleep(PAUSE);
    let early = child.try_wait().expect("status is readable");
    assert!(early.is_none(), "exited before Return with {early:?}");

    stdin.write_all(b"\n").expect("newline is written");
    let status = wait_with_deadline(&mut child);
    drop(stdin);

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout is piped")
        .read_to_string(&mut stdout)
        .expect("stdout is utf-8");
    assert!(status.success(), "exit status {status}");
    assert!(stdout.contains(TEXT), "shows the text");
}

/// Wait for the child to exit, killing it and failing the test once the deadline passes.
fn wait_with_deadline(child: &mut Child) -> ExitStatus {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("status is readable") {
            return status;
        }
        if start.elapsed() > DEADLINE {
            let _ = child.kill();
            panic!("no exit within {DEADLINE:?} of the newline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

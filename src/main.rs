//! Display "Hello, World" in the terminal for a fixed time, then exit.
//!
//! The first program of the progression toward an actor-model UI: a zero-crate TUI that owns the
//! alternate screen for a moment and restores the terminal on the way out.

use std::io::{self, Write};
use std::thread;

use uis_x1::{AltScreen, DISPLAY_TIME, TEXT};

/// Show the text on the alternate screen for the fixed time.
fn main() -> io::Result<()> {
    let _screen = AltScreen::enter()?;
    let mut out = io::stdout();
    out.write_all(TEXT.as_bytes())?;
    out.flush()?;
    thread::sleep(DISPLAY_TIME);
    Ok(())
}

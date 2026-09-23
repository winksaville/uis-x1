//! Display "Hello, World" in the terminal until Return is pressed, then exit.
//!
//! A step of the progression toward an actor-model UI: a TUI that owns the alternate screen and
//! the terminal's input until the user presses Return, and restores both on the way out.

use std::io::{self, Write};

use uis_x1::{AltScreen, RawMode, TEXT, wait_for_return};

/// Show the text on the alternate screen until Return.
///
/// Raw mode is entered first, so the alternate screen is never up while Ctrl-C can still kill
/// the process. The guards drop in reverse, so the screen is restored before the input settings.
fn main() -> io::Result<()> {
    let _raw = RawMode::enter()?;
    let _screen = AltScreen::enter()?;
    let mut out = io::stdout();
    out.write_all(TEXT.as_bytes())?;
    out.flush()?;
    wait_for_return(io::stdin().lock())
}

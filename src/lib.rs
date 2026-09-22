//! The fixed-time "Hello, World" display: what it shows, for how long, and the terminal guard
//! that owns the alternate screen while it shows, shared by the binary and its test.
//!
//! - Ctrl-C during the display is a known limit. The default SIGINT handler kills the process
//!   before the guard's `Drop` runs, so the alternate screen stays up. The next step's raw mode
//!   removes it.

use std::io::{self, Write};
use std::time::Duration;

/// How long the text stays on screen.
pub const DISPLAY_TIME: Duration = Duration::from_secs(1);

/// The text shown.
pub const TEXT: &str = "Hello, World";

/// Enter the alternate screen, clear it, home the cursor, and hide the cursor.
const ENTER_ALT_SCREEN: &str = "\x1b[?1049h\x1b[2J\x1b[H\x1b[?25l";

/// Show the cursor and leave the alternate screen, the inverse of `ENTER_ALT_SCREEN`.
const LEAVE_ALT_SCREEN: &str = "\x1b[?25h\x1b[?1049l";

/// The terminal while the program owns its alternate screen.
///
/// Created by `enter`, and dropping it restores the terminal, so a panic after entry still
/// leaves the terminal usable.
pub struct AltScreen;

impl AltScreen {
    /// Switch the terminal to the alternate screen and return the guard that switches back.
    pub fn enter() -> io::Result<AltScreen> {
        let mut out = io::stdout();
        out.write_all(ENTER_ALT_SCREEN.as_bytes())?;
        out.flush()?;
        Ok(AltScreen)
    }
}

impl Drop for AltScreen {
    /// Restore the terminal. A write error here has nowhere to go, so it is ignored.
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = out.write_all(LEAVE_ALT_SCREEN.as_bytes());
        let _ = out.flush();
    }
}

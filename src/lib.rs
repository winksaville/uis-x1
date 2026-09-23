//! The "Hello, World" display that stays until Return: what it shows, the terminal guards that
//! own the screen and the input while it shows, and the wait for Return, shared by the binary and
//! its tests.
//!
//! - Each guard restores its part of the terminal when dropped, so a panic after entry still
//!   leaves the terminal usable.
//! - Raw mode makes Ctrl-C a byte rather than a signal, so it can no longer kill the process
//!   before the guards restore the terminal.

use std::io::{self, BufRead, Write};

use rustix::termios::{self, OptionalActions, Termios};

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

/// The terminal's input settings while the program holds stdin in raw mode.
///
/// - Input arrives a byte at a time and unechoed, and Ctrl-C and Ctrl-Z arrive as bytes rather
///   than signals.
/// - Output processing is off too, so a written newline no longer returns the cursor.
/// - Created by `enter`, and dropping it restores the saved settings.
pub struct RawMode {
    /// The settings in force before `enter`, restored on drop.
    saved: Termios,
}

impl RawMode {
    /// Put stdin's terminal in raw mode and return the guard that restores it.
    ///
    /// Returns `None` when stdin is not a terminal, a pipe for instance, since it has no terminal
    /// settings to change.
    pub fn enter() -> io::Result<Option<RawMode>> {
        let stdin = io::stdin();
        if !termios::isatty(&stdin) {
            return Ok(None);
        }
        let saved = termios::tcgetattr(&stdin)?;
        let mut raw = saved.clone();
        raw.make_raw();
        termios::tcsetattr(&stdin, OptionalActions::Now, &raw)?;
        Ok(Some(RawMode { saved }))
    }
}

impl Drop for RawMode {
    /// Restore the saved settings. An error here has nowhere to go, so it is ignored.
    fn drop(&mut self) {
        let _ = termios::tcsetattr(io::stdin(), OptionalActions::Now, &self.saved);
    }
}

/// Read bytes until a CR or LF, or until the input ends, and ignore every other byte.
///
/// - A terminal in raw mode sends CR for Return, and a pipe sends LF.
/// - Ctrl-C arrives here as a byte under raw mode and is ignored like any other key.
/// - The end of input returns too, since a closed input can never deliver a Return.
/// - The input is buffered, so reading it a byte at a time costs no system call per byte.
pub fn wait_for_return<R: BufRead>(input: R) -> io::Result<()> {
    for byte in input.bytes() {
        if matches!(byte?, b'\r' | b'\n') {
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The wait's handling of each kind of byte, fed from a slice with no terminal involved.

    use super::*;

    /// Ctrl-C, as the byte raw mode delivers.
    const CTRL_C: u8 = 0x03;

    /// A CR or an LF ends the wait and leaves the bytes after it unread.
    #[test]
    fn ends_at_cr_or_lf() -> io::Result<()> {
        for end in *b"\r\n" {
            let bytes = [b'a', end, b'z'];
            let mut input = &bytes[..];
            wait_for_return(&mut input)?;
            assert_eq!(input, b"z", "after {end:#04x}");
        }
        Ok(())
    }

    /// A Ctrl-C does not end the wait.
    #[test]
    fn ignores_ctrl_c() -> io::Result<()> {
        let bytes = [CTRL_C, CTRL_C, b'\r', b'z'];
        let mut input = &bytes[..];
        wait_for_return(&mut input)?;
        assert_eq!(input, b"z");
        Ok(())
    }

    /// The end of input ends the wait with no Return seen.
    #[test]
    fn ends_at_end_of_input() -> io::Result<()> {
        let mut input: &[u8] = b"abc";
        wait_for_return(&mut input)?;
        assert!(input.is_empty());
        Ok(())
    }
}

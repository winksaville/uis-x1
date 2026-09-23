# Terminal host

What the terminal host has to know about the terminal's input and output modes, found in the cycle
`feat: quit on return`, which added raw mode, and in the questions around it.

## Normal mode keeps keys for itself

In normal mode the terminal driver acts on a set of keys and never passes them to the program, and
it delivers nothing until Return or Ctrl-D, then a whole line. The Linux defaults:

- Signals: Ctrl-C sends SIGINT, `Ctrl-\` sends SIGQUIT, and Ctrl-Z sends SIGTSTP.
- Line editing: Backspace erases a character, Ctrl-U erases the line, and Ctrl-D ends input, a
  zero-byte read at the start of a line.
- Extended editing: Ctrl-W erases a word, Ctrl-V passes the next key through literally, and Ctrl-R
  redraws the line when echo is on.
- Flow control: Ctrl-S pauses output and Ctrl-Q resumes it.
- Translation: Return arrives as LF, since the CR to LF translation is on.
- Other systems: on macOS and the BSDs, Ctrl-T prints a status line, Ctrl-Y is a delayed suspend,
  and Ctrl-O discards output. Linux passes all three through.

## Raw mode and what it costs

The host enters raw mode with the standard settings, those of `cfmakeraw`, so every key that sends
a byte arrives as that byte, one at a time and unechoed.

- Return arrives as CR, while a pipe sends LF, so a reader accepts both.
- Output processing is off as well, so a written LF moves down without returning the cursor. The
  ANSI presenter writes CR LF, or positions the cursor explicitly.
- Raw mode applies only when stdin is a terminal, since a pipe has no terminal settings.

## What never arrives, even in raw mode

- The terminal and the desktop take their own shortcuts first, whatever the mode: copy and paste,
  scrollback, tabs, and window switching.
- Modifier keys pressed alone send no byte.
- Arrows, function keys, Home, and End arrive as escape sequences, and Alt plus a key arrives as
  ESC then the key. A lone Esc is told from the start of a sequence only by waiting.
- Some keys share a byte: Tab and Ctrl-I, Enter and Ctrl-M, Esc and `Ctrl-[`, and Ctrl-Shift-A and
  Ctrl-A.

## Key press, repeat, and release

The classic terminal protocol sends characters, so a host sees presses only. A held key repeats as
more of the same bytes, and a release sends nothing. We think a key message should carry its kind,
press, repeat, or release, with a classic terminal host only ever sending press.

- The kitty keyboard protocol reports press, repeat, and release, modifier keys alone, and the keys
  that otherwise share a byte. The program turns it on with an escape sequence, and the guard that
  turns it on turns it off again.
  - Support depends on the terminal, and kitty, foot, WezTerm, Ghostty, Alacritty, and iTerm2
    have it.
  - The crossterm crate speaks it and can query whether the terminal does.
- The Windows console input API reports key down and key up natively.
- The winit window events carry pressed or released, a repeat flag, and the physical and logical
  key, on every platform, so the GUI host has all three kinds.
- The Linux text console's raw scan codes and direct reads of the input devices both report
  releases, and neither fits. The first works only on a real virtual console, not in a terminal
  window, and the second needs extra permissions and sees keys typed into every window.

## Windows

Windows has no POSIX terminal settings, so the terminal module of rustix is Unix only, and a
Windows host needs its own console module behind a platform switch.

- On the input handle it clears line input, echo, and processed input, the last so that Ctrl-C
  arrives as a byte.
- On the output handle it turns on virtual terminal processing, without which the escape sequences
  are not honored.
- The crossterm crate does both, and is the one-crate alternative when a Windows terminal build
  becomes a goal.

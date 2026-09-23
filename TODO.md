# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record, read first at
acquaint, acted on, and reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's shape is
the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in [The In Progress
block](agent-data/notes.md#the-in-progress-block).

_No cycle currently in progress._

## Waiting

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Use the [Prose
form](agent-data/prose.md#prose-form).

### The GUI twin

The same program as a window: winit, softbuffer, and a bitmap font blitted into a grid-shaped
buffer, so the later cell-grid presenter is a refactor rather than a rewrite.

### Record the actor access rule

Write into the notes the rule that an actor is reached only through its message interface, never
by looking at its state through a backdoor, and the one exception. Deployment on one thread is a
fact tests and code may not rely on, so debugging goes through query messages the actor chooses to
answer, and tests assert on replies. The exception is an actor's own unit tests in its module,
which drive it with messages and may then inspect its private fields, since Rust privacy makes that
module the only place that can. Fields are never made public for a test.

### Actors and messaging

Rebuild the TUI app on an executor over the zc-ring rings, with app, label, display, timer, and
supervisor actors and stdin as a device thread, per [actor-model-1.md](notes/actor-model-1.md).

### TUI or GUI backend

The core emits a cell grid and the ANSI and pixel presenters consume it, the claim in
[actor-model-1.md](notes/actor-model-1.md) to verify.

## Ideas

_None._

## Bugs

_None._

## Closed

### feat: quit on return

#### Problem

The display ends on a timer, so the user cannot choose when to leave it. A Ctrl-C during the
display kills the process before the guard's `Drop` runs, so the alternate screen stays up.

#### Solution

Stdin's terminal goes into raw mode through the rustix crate, and the display stays until CR or LF
is read from stdin, or until the input ends. Raw mode turns Ctrl-C into a byte the wait ignores.

- The raw-mode guard applies only when stdin is a terminal and restores the saved settings on drop,
  and the binary enters it before the alternate screen.
- The wait is a library function over a buffered reader, unit tested with byte slices.
- The integration test pipes stdin, checks that the binary waits, then sends a newline and checks
  the exit and the text.
- The findings about terminal input went into [terminal-host.md](notes/terminal-host.md).
- The workspace gained a README and the MIT and Apache-2.0 license files.

#### Acceptance check

`cargo test` passes with a test that runs the binary with stdin piped, finds it still running after
a pause, writes a newline, and sees a clean exit with the text on stdout. In a real terminal, a
Ctrl-C during the display is ignored, Return ends it, and the shell comes back with its echo and
line editing working.

Result: pass. `cargo test` passes, and the terminal half ran on a pseudo-terminal. Two Ctrl-Cs half
a second apart left the program running, Return ended it with exit status 0, no `^C` was echoed,
and the terminal settings read before and after were identical. A control run in the same harness
showed a Ctrl-C killing a plain `sleep`, so the harness delivers real signals.

#### Ladder

- feat: quit on return (done)

#### Deliberation

- Raw mode through rustix: the crate's safe calls read the terminal settings, make them raw, write
  them back, and check whether stdin is a terminal.
  - It pulls in three crates on Linux and makes the system calls itself there, without libc.
    Since crossterm is built on it, a later move to crossterm starts from the same base.
  - Calling libc directly was the first plan, and it makes every call `unsafe`.
  - The termion crate applies raw mode to the output handle, and the test pipes stdout, so
    entering raw mode fails there.
  - The crossterm crate pulls in 13 to 27 crates for raw mode plus output commands, and the plan
    builds the output layer itself as the cell-grid presenter.
- Windows later: Windows has no POSIX terminal settings, so the terminal module of rustix is Unix
  only, and macOS works unchanged.
  - A Windows console module comes behind a platform switch when a Windows terminal build is a
    goal. It clears line input, echo, and processed input, and turns on virtual terminal
    processing so the escape sequences are honored.
  - The one-crate alternative at that point is crossterm.
- The standard raw settings: the `make_raw` call gives the settings of `cfmakeraw`, which clear
  more than line input, echo, and signals.
  - Return arrives as CR, since the CR to LF translation is off.
  - Output processing is off, so a written newline no longer returns the cursor. The program
    writes no newline, and the cell-grid presenter will write explicit carriage returns.
  - Ctrl-Z and `Ctrl-\` arrive as bytes like Ctrl-C, and flow control by Ctrl-S and Ctrl-Q is off.
- CR or LF ends the display: a terminal in raw mode sends CR for Return, and a pipe sends LF.
- Raw mode only on a terminal: a pipe has no terminal settings, and the test feeds stdin through
  one.
- End of input ends the display: a closed stdin can never deliver a Return, so waiting past its end
  would hang the program.
- Raw mode before the alternate screen: entering raw mode first leaves no moment when the
  alternate screen is up and Ctrl-C still kills the process. The guards drop in reverse, so the
  screen is restored before the input settings.
- The wait in the library: it is a function over a buffered reader, so a unit test feeds it
  bytes, Ctrl-C's included, with no terminal.
- Stdin held open until the exit: the test keeps its end of the pipe open while it waits, since
  closing it would end the display through end of input and hide a broken Return.
- A buffered reader for the wait: clippy flags reading a byte at a time from an unbuffered reader,
  and stdin's lock and a byte slice are both buffered, so the wait takes a buffered reader.
- A pseudo-terminal for the terminal half: `script` runs the binary on one, so the Ctrl-C and
  Return bytes pass through a real terminal driver, which the sandbox otherwise lacks.
- README and licenses in this commit: they arrived during the cycle, and the user chose this
  commit over a Todo entry of their own, as [Unplanned work](AGENTS.md#unplanned-work) allows.
  - The licenses match the sibling projects' dual MIT and Apache-2.0 pair, with this project's
    year.

# References

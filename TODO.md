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

### Quit on Return

Display until CR or LF is read from the keyboard, which needs raw mode and turns Ctrl-C into an
ignored byte.

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

### feat: display hello world for a fixed time

#### Problem

The workspace has no application, and the planned progression toward an actor-model UI with TUI
and GUI backends needs a first program that does something visible, the baseline the actor version
is later measured against.

#### Solution

A zero-crate TUI that enters the alternate screen, hides the cursor, shows "Hello, World" for a
fixed number of seconds, and restores the terminal on the way out. The cycle also gives the
workspace what the set assumes: this record, a version-of-record, a notes index, and cargo
validation.

#### Acceptance check

`cargo test` passes with a test that runs the binary with stdout captured and finds the text, with
exit status 0, and the run takes at least the fixed time.

Result: pass. The check was narrowed during review from checking the escape sequences in order to
checking the text alone, the deliberation says why.

#### Deliberation

- TUI, not GUI: the terminal version needs no crates and no display server, and the GUI twin is a
  later step of the progression.
- Fixed time, no argument: the progression starts with a TUI, not a CLI, so the seconds are a
  constant and the next step replaces the timer with the Return key.
- Ctrl-C limit accepted: the default SIGINT handler kills the process before Drop runs, so a Ctrl-C
  mid-sleep leaves the alternate screen up. Raw mode in the next step turns Ctrl-C into an ignored
  byte.
- Restore in Drop: a panic then still restores the terminal, and the guard is the seed of the
  terminal host's enter and leave.
- One-second display: the acceptance check runs the real binary for its fixed time, so the time
  is what `cargo test` waits. One second is long enough to see and short enough to test.
- Library plus binary: an integration test cannot import from a binary, so the constants and the
  guard live in the library and the binary and the test both use them.
- Smoke test, not screen scraping: the test checks that the text appears, the exit is clean, and
  the run takes its time, and nothing about the escape sequences. Scraping output is the wrong
  model once a GUI with overlapping windows exists, so the tested layer becomes the cell grid
  when it arrives, per [actor-model-1.md](notes/actor-model-1.md).

#### Ladder

- feat: display hello world for a fixed time (done)

# References

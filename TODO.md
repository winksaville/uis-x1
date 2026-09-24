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

### feat: workspace-gui-softbuffer

#### Problem

The project is one package holding the terminal program, and the progression needs a window twin
of it, with more twins of each kind expected. A window stack pulls in some 86 crates on Linux,
which a single package would make every terminal build compile.

#### Solution

Make the repo a Cargo workspace whose root manifest is only the workspace, move the terminal
program unchanged into a `tui-rustix` member, and add a `gui-softbuffer` member that shows the text
in a window until Return. The window twin draws with winit and softbuffer, and a bitmap font
renders the text into the pixel buffer.

- Each member is named for its stack, and its package and binary carry that name alone,
  `tui-rustix` and `gui-softbuffer`.
- The version-of-record moves to the workspace manifest, and both members inherit it.
- The text scales by the window's scale factor in whole steps.
- Validation installs each binary, and the stale `uis-x1` binary is uninstalled.

#### Acceptance check

`cargo test` at the root runs both members' tests and passes. The terminal twin's tests pass
unchanged apart from names, and a window-twin test renders the frame into a pixel buffer and finds
the text's pixels in it. The terminal twin passes the pseudo-terminal check from the last cycle
again. In a desktop session, `gui-softbuffer` opens a window showing the text, and Return or closing
the window exits it.

#### Deliberation

- Workspace-only root: the root manifest declares the workspace and no package, so every cargo
  command at the root covers every member.
  - Two binaries in one package would share the window stack's crates unless a cargo feature gated
    them, and the gating is friction on every command.
  - Keeping the terminal program as the root package, with the window twin as a member, moves no
    files. But cargo commands at the root reach only the root package, and the later core crate
    would sit lopsided beside it.
- Members named for their stack: more twins of each kind are expected, `tui-ratatui` for one, so a
  member's name says what differs.
- Bare stack names on packages and binaries: a member's package and binary are its directory's
  name, which is simpler to type and to read, chosen by the user at the move's review.
  - The opening planned a project prefix, `uis-x1-tui-rustix`, since `cargo install` puts every
    binary in the shared cargo bin directory, where a bare stack name could collide with another
    project's binary. That collision is the cost accepted.
  - A short package with a prefixed binary, through a `[[bin]]` name, kept the short cargo commands
    and the collision guard, and was declined for the simpler manifest.
- Softbuffer for the window twin: winit is common to nearly every Rust GUI stack, egui and iced
  included, so the software pixel path is what sets this twin apart.
  - The crate is mature, its main branch is active, and its latest release is from 2025-12-13.
  - It is the thinnest layer to the OS, and a pixel presenter over it later runs unchanged over a
    bare-metal framebuffer, per [actor-model-1.md](notes/actor-model-1.md).
- The stable winit line: the next line is still in beta with a changed API, so the twin starts on
  the stable one.
- Fonts from embedded-graphics: its mono fonts are bitmap fonts with a draw-target trait, the
  pairing the actor notes plan for one pixel presenter over a window and a framebuffer, and it
  adds six small crates.
  - The font8x8 crate is lighter, one crate with one 8 by 8 font, and has no draw-target trait.
- Whole-step scaling: a 10 by 20 bitmap font is tiny on a high-density display, so the text scales
  by the window's scale factor rounded to a whole number, which keeps each font pixel a sharp
  square.
- Return or window close exits: the window twin mirrors the terminal twin's Return, and a window
  can also be closed from its title bar, so both end it. Escape stays unbound, as in the terminal
  twin.
- The text copied, not shared: each twin keeps its own copy of the text, since a shared crate for
  one constant is premature and the core crate arrives with the cell grid.
- The move as its own rung: the terminal program moves unchanged in a rung of its own, so the move
  is reviewed as a move before any window code lands on it.
- A pixel-buffer test for the window twin: the sandbox has no display, so the test renders into a
  buffer and checks pixels, and only the window itself is checked by hand.
- No restart between cycles: the user opened this cycle in the session that ran the last one.
- The last cycle lands first: main advances to the quit-on-Return commit and its bookmark is
  deleted before this opening's push, so that commit stays out of this ladder.

#### Ladder

- [feat: workspace-gui-softbuffer opening][1] (done)
- [refactor: move the tui into the workspace][2] (done)
- [feat: add the gui-softbuffer twin][3]
- [feat: workspace-gui-softbuffer closing][4]

##### feat: workspace-gui-softbuffer opening

The cycle's setup commit: create and publish the bookmark, empty `## Closed`, move the Todo entry
into this block, and bump the version-of-record.

The plan settled at the opening: a workspace-only root, members named for their stack with
prefixed packages and binaries, softbuffer on the stable winit line, embedded-graphics fonts scaled
in whole steps, and exits on Return or window close.

##### refactor: move the tui into the workspace

The terminal program is the root package, so a second program cannot sit beside it without sharing
its dependencies. The root becomes a workspace-only manifest, and the program moves unchanged into
`tui-rustix/` under that name.

- The root declares the workspace and the shared `version` and `edition`, and the member inherits
  both, so the version-of-record has one home for every member.
- The package, and with it the binary and the library crate, is `tui-rustix`, the bare stack name
  the review chose over the planned prefix, and the sources change only where they name the crate
  or the binary.
- The member keeps its own clippy lints for now. Hoisting them to the workspace waits for a second
  member to share them.
- Validation installs from `tui-rustix/`, since `cargo install --path .` needs a package at the
  root, and the stale `uis-x1` binary is uninstalled.

##### feat: add the gui-softbuffer twin

The progression has no window program. A `gui-softbuffer` member opens a window with winit, draws
the text with a bitmap font into a softbuffer pixel buffer, and exits on Return or when the window
closes.

##### feat: workspace-gui-softbuffer closing

Closing out the cycle.

## Waiting

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing is moving an entry. Each is a
`###` heading, so a citation is a link to its anchor. Use the [Prose
form](agent-data/prose.md#prose-form).

### Commit the config file form entry

The agent-files name the workspace config `.vc-config.md`, and this work-repo's is
`.vc-config.toml`. The custom.md entry that reads one as the other was set aside during the
workspace-gui-softbuffer cycle, since an agent-file change is its own cycle.

- Add the entry below to custom.md's `## Project conventions and overrides`, first in its list,
  as its own cycle. `tmp/custom-md.patch` holds the same text as a patch while `tmp/` survives.
- Retire it when [Propose the two config file forms to the
  set](#propose-the-two-config-file-forms-to-the-set) lands in the payload and is adopted.

```markdown
- Config file form: the workspace config is either `.vc-config.toml`, plain TOML, or
  `.vc-config.md`, whose fences tagged `toml` hold it, and this work-repo uses the first.
  Wherever the agent-files say `.vc-config.md`, read it as whichever form the repo has.
  Supersedes the file name in [The dual-repo model](AGENTS.md#the-dual-repo-model) and
  [.vc-config.md](agent-data/jj.md#vc-configmd).
```

### Propose the two config file forms to the set

The set's [.vc-config.md](agent-data/jj.md#vc-configmd) section names one file form, and vc-x1
reads two, a plain `.vc-config.toml` and a `.vc-config.md` whose `toml` fences hold the config.
Propose that the section name both, so no adopter needs a custom.md entry for it.

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

_None._

# References

[1]: #feat-workspace-gui-softbuffer-opening
[2]: #refactor-move-the-tui-into-the-workspace
[3]: #feat-add-the-gui-softbuffer-twin
[4]: #feat-workspace-gui-softbuffer-closing

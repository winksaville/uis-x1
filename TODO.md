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

The project is one package holding the terminal program, and the progression needs window twins of
it in two loop shapes, one where the toolkit owns the loop, as desktops require, and one where the
program owns it, as bare metal does, with more twins of each kind expected. A window stack pulls in
some 100 crates on Linux, which a single package would make every terminal build compile.

#### Solution

Make the repo a Cargo workspace whose root manifest is only the workspace, move the terminal
program unchanged into a `tui-rustix` member, and add two window members that show the text until
Return: `gui-winit-softbuffer`, where winit owns the loop and softbuffer presents the pixels, and
`gui-minifb`, which owns its loop. A bitmap font renders the text into a pixel buffer, in a library
member both window twins share.

- Each member is named for every layer of its stack that differs, and its package and binary carry
  that name alone.
- The version-of-record moves to the workspace manifest, and every member inherits it.
- The text scales by the window's scale factor in whole steps.
- Validation installs each binary, and the stale `uis-x1` binary is uninstalled.

#### Acceptance check

`cargo test` at the root runs every member's tests and passes. The terminal twin's tests pass
unchanged apart from names, and the shared renderer's tests render the frame into a pixel buffer
and find the text's pixels in it. The terminal twin passes the pseudo-terminal check from the last
cycle again. In a desktop session, `gui-winit-softbuffer` and `gui-minifb` each open a window
showing the text, and Return or closing the window exits it.

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
  - The opening named the window twin `gui-softbuffer`, since winit is common to nearly every Rust
    GUI stack. With a twin that has no winit to follow, the name spells out both layers,
    `gui-winit-softbuffer`, chosen by the user at the window twin's review.
- Two window twins in one cycle: winit owns the loop, the shape desktop toolkits impose and the
  one the actor model must fit on Windows and macOS, while minifb leaves the loop to the program,
  the shape bare metal takes. The user added the second at the window twin's review, as a rung
  rather than a Todo.
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
- The text copied, not shared: the terminal twin keeps its own copy of the text, since a shared
  crate for one constant is premature and the core crate arrives with the cell grid.
- The renderer shared at its second user: the minifb twin would otherwise copy the canvas, the
  frame, and their tests, so the renderer moves into a library member both window twins use, the
  start of the pixel presenter the actor notes plan. Chosen by the user over a copy.
- The moves as their own rungs: the terminal program, and later the renderer, move unchanged in
  rungs of their own, so each move is reviewed as a move before new code lands on it.
- A pixel-buffer test for the window twin: the sandbox has no display, so the test renders into a
  buffer and checks pixels, and only the window itself is checked by hand.
- No restart between cycles: the user opened this cycle in the session that ran the last one.
- The last cycle lands first: main advances to the quit-on-Return commit and its bookmark is
  deleted before this opening's push, so that commit stays out of this ladder.

#### Ladder

- [feat: workspace-gui-softbuffer opening][1] (done)
- [refactor: move the tui into the workspace][2] (done)
- [feat: add the gui-winit-softbuffer twin][3] (done)
- [refactor: share the pixel renderer][4]
- [feat: add the gui-minifb twin][5]
- [feat: workspace-gui-softbuffer closing][6]

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

##### feat: add the gui-winit-softbuffer twin

The progression has no window program. A `gui-winit-softbuffer` member opens a window with winit,
draws the text with a bitmap font into a softbuffer pixel buffer, and exits on Return or when the
window closes.

- The drawing is the library's and the window is the binary's: the library draws the frame into
  any slice of pixels, so the tests and a later framebuffer presenter use it with no window.
- The canvas is an embedded-graphics draw target that turns each font pixel into a square of
  whole screen pixels, and embedded-graphics clips at the canvas's size in font pixels.
- The text starts at the top left corner, where the terminal twin's cursor starts, which is also
  where a cell grid's first cell will sit.
- A handler cannot return an error in winit's application trait, so the first error is kept and
  ends the event loop, and `main` returns it.
- The tests check that the frame holds only the text and background colors with the text inside
  its box, that scale 2 is scale 1 with each pixel doubled, and that a clipped frame is the top
  left corner of the unclipped one.
- The terminal member still builds on its own four crates, and the window member brings some 105.
- The member was `gui-softbuffer` until its review renamed it for both layers, the next rungs'
  twin having no winit.

##### refactor: share the pixel renderer

The minifb twin needs the same canvas and frame as the winit twin. They move unchanged, tests
included, from the winit twin's library into a library member both window twins depend on.

##### feat: add the gui-minifb twin

The progression has no window program that owns its loop. A `gui-minifb` member opens a window
with minifb and runs its own loop, drawing the frame through the shared renderer and presenting it,
until Return or a close.

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
[3]: #feat-add-the-gui-winit-softbuffer-twin
[4]: #refactor-share-the-pixel-renderer
[5]: #feat-add-the-gui-minifb-twin
[6]: #feat-workspace-gui-softbuffer-closing

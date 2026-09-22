# Actor model 1: TUI vs GUI and the Final Application Architecture

Captured from a Claude Code session on 2026-09-22 in `uis-x1`. The user asked for thoughts only, so
no code was written. The thoughts are reasoning rather than measurement: read every mechanism and
prediction below as a "We think ..." claim. To ground the answers the bot read three sources:

- Rings and pool: `../../zc-ring-x1` holds the `no_std` SPSC (v3) and MPSC (v2) rings over
  caller-provided memory, zero-copy typed messages via zerocopy, and `pool::v0`. Development is
  moving to `../../zc-msg-x1`, which is pre-seed with docs only.
- Actor design intent: `../../actor-discussions.md` records actors as state machines, actors that
  never block, senders that never block, explicit drop-on-exhaustion back-pressure, a ring plus an
  overflow list, and the open axes of pool ownership, message typing, priority, timers, and
  supervision.
- Runtime sketch: `../../actor-x1/crates/actor-x1/notes/design.md` has the handler running to
  completion and never blocking, expensive work on spawned threads, future HSM actors, and a runtime
  with a management system.

## Summary

- Initial app: a TUI with zero crates, one print of escape codes plus the text, a ten second sleep,
  and one print to restore.
- The FAA needs both presenters, since phones have no terminal and bare metal has no window system.
- Cheapest agnostic design: the core emits a fixed-size cell grid, one presenter turns it into ANSI
  bytes, and another blits it to pixels. Decide cells versus widget tree before writing FAA code.
- Runtime: executors are the unit of threading, with one inbox ring per executor, global actor ids,
  and a startup table mapping actor to executor. Device threads and workers hold all blocking work.
- UI elements as actors works with three rules: a device, element, and service taxonomy, containers
  are actors and items are data, and an epoch on every multi-hop protocol.
- Build a deterministic sim host, one executor with a virtual clock and scripted input, before the
  TUI host once the FAA begins.

## Round 1: TUI vs GUI

### Round 1 constraints (from user)

> I want to compare TUI and GUI, constraints:
> - Pure Rust
> - Final Application Architecture FAA
>   - Message based zcr-mpsc-v2 and zcr-spsc-v3
>   - The app uses pre-allocated memory using Pool
>   - Portable between linux, windows, mac, android, iphone, bare metal
>   - Agnostic between TUI or GUI
> - Initial application does not use FAA
>   - Renders "Hello World" for 10 seconds and exits.
>   - Linux
>   - TUI or GUI whichever is simpler
>
> Thoughts only

### Round 1 thoughts

Short version: the initial app should be a TUI with zero crates. The final architecture needs both
presenters regardless, because phones have no terminal and bare metal has no window system. The
cheapest way to be agnostic is a core that emits a fixed-size cell grid, with one presenter that
turns it into ANSI bytes and another that blits it to pixels. The decision to make before any FAA
code is the abstraction level, cells or widgets, since it sets everything downstream. The spsc v3
and mpsc v2 rings in zc-ring-x1 are `no_std` and move typed messages in place over caller-provided
memory, which grounds the channel comments below.

#### Initial app: TUI wins by a wide margin

- The entire program is one print of escape codes plus the text, a ten second sleep, and one print
  that restores the screen. It needs no crates, compiles near-instantly, and runs over SSH, in CI,
  or on a serial console.
- The GUI equivalent with eframe or iced pulls in hundreds of crates, needs a display server, and
  the exit requires an event-loop wakeup at the deadline instead of a sleep.
- A winit plus softbuffer version stays closer to the constraints but is far more code, since you
  handle resize, redraw, and glyph drawing yourself.
- The one thing GUI does simpler is cleanup. The OS destroys the window on exit, while the TUI must
  restore the terminal, and a Ctrl-C during the sleep leaves the alternate screen active unless
  handled.

#### How the constraints score

| Constraint | TUI | GUI |
|---|---|---|
| Pure Rust | Bytes to stdout, and crossterm and ratatui are pure Rust. | Rust stack ends in FFI to the OS for a window and display. Software rendering through softbuffer keeps that layer thinnest. |
| Pool memory | Fixed cell grid, fixed event structs, genuinely zero heap. | Every toolkit allocates. Only a hand-rolled framebuffer presenter can be allocation free. |
| Linux, Windows, Mac | Yes. Windows needs VT mode switched on, which crossterm does. | Yes, through winit. |
| Android, iPhone | No terminal exists. TUI on mobile means a window rendering your grid. | The only option. Packaging as an APK or Xcode bundle is the real cost, not rendering. |
| Bare metal | Best case. UART plus ANSI escapes is the classic embedded console. | Framebuffer plus embedded-graphics, which is `no_std` and allocation free. |
| Message fit | Natural. A single-threaded loop works. | The toolkit owns the main thread on Mac, iOS, and Android. The core must run as a step function inside the loop or on its own thread. |

#### The design that falls out

- The cell grid is the intersection of every target. For terminals and serial it becomes ANSI
  bytes. For windows, phones, and bare-metal displays it becomes pixels through a monospace bitmap
  font. The embedded-graphics crate supplies both the fonts and a draw-target trait, so one pixel
  presenter runs unchanged over a softbuffer window and a memory-mapped framebuffer.
- Zero-copy frames fit the rings. A grid diff or a whole grid can live in a pool slot that the
  presenter reads in place. On a UART you want diffs anyway, because a full styled redraw at serial
  speed is visibly slow.
- Waiting belongs to the host, never the core. A `no_std` ring cannot block, so the core should be
  a non-blocking step: drain the mpsc, update state, write the frame, return. The terminal host
  waits on stdin plus a timer, the window host waits inside winit and wakes it through its proxy,
  and bare metal waits in WFI while ISRs push into the rings.
- The Linux TUI host is the bare-metal shape with std: one loop, one thread. Building the FAA on it
  first means mirroring the hardest target from day one, and the GUI host later adds only the
  cross-thread wakeup.
- Workspace shape: a `no_std` core crate with messages, pool, state, and grid, a `no_std` ANSI
  presenter, a `no_std` pixel presenter, thin std hosts for terminal and window, and target crates
  for bare metal. The core never sees crossterm, winit, or a UART register.
- The fork to pick first: cells are cheap, everything works, and tests assert on a grid, but mobile
  looks like a terminal and the soft keyboard is awkward. A widget tree gives native-looking mobile
  at several times the work, with the TUI as the lossy backend. A third route is a headless core
  that sends domain state messages and lets each host write its own view, which avoids the
  abstraction problem by duplicating view code.
- If TUI were ever dropped, Slint covers desktop, mobile, and MCUs from one description. It is a DSL
  rather than plain Rust, needs an allocator, and has license terms for embedded use, so it does not
  fit as stated.
- Off-the-shelf shortcut on desktop: ratatui's buffer and backend trait are this exact abstraction,
  and community backends render it into egui or a software pixel buffer. It allocates, so it belongs
  in a host, not the core.

Things that bite later:

- Terminal input is lossy with no key-release events, so the core must not depend on key-up.
- Keep colors as a named-plus-RGB enum and let the ANSI presenter downgrade.
- Wide characters need a width table once you leave ASCII.
- The mobile host spends its effort on packaging and the soft keyboard, not on drawing.

## Round 2: A little deeper into FAA

### Round 2 constraints (from user)

> A little deeper into FAA:
> - multi-threaded
> - application are composed of actors
> - actors communicate via messages and process messages to completion
>   - they can be on a thread by themselves or with many others
>   - are agnostic as to what thread they run on
>   - if there are time constraints they maybe on their own thread
>   - expensive work is processed outside of the actor model
>   - no async/await
>
> I envision each input/output element of the UI to be an actor

### Round 2 thoughts

These constraints describe an active-object runtime in the Quantum Platform family: a few executor
threads, each draining one inbox and dispatching to the actors it owns, with all blocking pushed
out to device threads and workers. That fits the UI-element-as-actor idea well, and it maps
directly onto the ring, overflow list, and pool work in zc-ring-x1 and the actor-discussions notes.
Three refinements make it work in practice: a taxonomy of device, element, and service actors, a
rule that containers are actors and items are data, and an epoch on every multi-hop protocol so
reordering cannot corrupt a frame. The TUI versus GUI question then shrinks to two device actors
and one presenter thread.

#### Runtime shape the constraints imply

- Executors are the unit of threading, not actors. There is one inbox ring per executor, a global
  actor id in every message, and a startup table mapping actor to executor. Moving a time-critical
  actor onto its own thread is one row in that table, which is exactly what thread agnosticism
  buys.
- Three kinds of threads:
  - Executors run handlers.
  - Device threads block on stdin, a UART, or the window loop, and only produce or consume
    messages.
  - Workers take job messages, do the expensive work, and reply with a result carrying the caller's
    correlation id.
- On bare metal, device threads become ISRs and workers become an idle-priority loop or DMA.
  Nothing else changes, which is the point of the taxonomy.
- The ring never blocks, but the executor must park when idle. Spinning is right for benches and
  wrong for a phone or a laptop. Wake only on the empty to non-empty transition, with a sleeping
  flag the producer checks after enqueue, so the measured fast path survives under load and idle
  costs only a wake syscall.
- The park primitive is the one host-specific piece: futex or condvar on std, poll on stdin plus an
  eventfd for a single-threaded TUI host, the winit proxy for a window, and WFI on bare metal. The
  rings stay portable.
- Timers belong to the executor, because the park deadline is exactly the next expiry. Use a
  fixed-capacity table with the clock injected through a trait. The UI needs timers at once: the
  lone ESC ambiguity in terminal input, key repeat, cursor blink, double click, animation, and the
  ten second exit.
- Local sends can skip atomics through a thread-local queue, but that weakens causal order between
  local and remote paths and can starve remote senders. Start with one inbox, pay the atomic, and
  measure before adding the fast path.
- Priority for an ISR-fed or deadline actor sharing an executor is simplest as a second ring drained
  first with a batch bound, rather than reordering within one ring.
- The HSM ambition and UI widgets converge. Button, text field, and dialog state machines are the
  textbook hierarchical state machine examples, so elements are the best proving ground for HSM
  actors.

#### UI elements as actors

| Kind | Examples | Host specific | Notes |
|---|---|---|---|
| Device | keyboard decoder, mouse decoder, resize, display composer | Yes | Pure state machines fed by device threads. The terminal decoder needs the ESC timer, and the window decoder is trivial. |
| Element | label, button, text field, list, panel, dialog | No | One actor per element with input and render in the same actor. A button is not two actors. |
| Service | layout, router, timer, supervisor | No | Layout owns the tree as data. Router owns focus and hit-testing. Supervisor owns startup and quit. |

- Granularity rule: an element is an actor when it has state that evolves through events. Static
  decoration is data inside its container.
- Containers are actors, items are data. A list with many rows is one actor that virtualizes, never
  one actor per row. This bounds the actor count at build time, which the pools require.
- Layout as a protocol: full measure then arrange costs several hops per element per pass, cheap on
  one executor, but it needs an epoch so a stale reply from before a resize cannot land in the new
  frame.
- Cheaper layout: elements register size preferences once and resend only when content changes
  size, so Layout computes with no measure round trip. Cell-based UIs rarely need more.
- Frame coherence: Display owns a back grid and presents only on epoch completion or a tick, never
  mid-update. The presenter thread receives frames over spsc v3 with a keep-latest policy, which is
  the explicit drop applied where dropping is correct.
- Routing: elements never see raw device events. Router forwards keys along the focus chain and
  hit-tests pointer events against rects Layout publishes each epoch. Modal dialogs are a capture
  stack in Router, and z-order is layer data Display composes in order.
- Coalesce time: blink, repeat, and animation subscribe to one tick rather than owning timers.
  Every feedback loop, such as content growing and forcing relayout, must close through a tick or
  an epoch, or you get a livelock in place of the deadlocks the model removed.
- The cells versus widget-tree fork now lives entirely in Layout's unit system and Display's
  composition. Element input logic is identical either way, so the fork is contained.

#### Message-level consequences

- Fixed-size messages live in the ring slot, as the ring already does, and are freed when the
  consumer advances. Only large payloads such as a rendered block, text content, a paste, or a
  frame go out of line in pool slots by index.
- That settles pool ownership for the UI. Producer-owned pools with a return-to-owner message from
  the consumer need no cross-thread free list, and same-executor consumers free directly.
- Overflow policy attaches to the message class, not the ring. Pointer motion coalesces to latest.
  Frames keep latest. Key presses are sized for worst case, with paste routed as one payload rather
  than a burst of key events. Quit and focus must never drop, so size for it and assert in debug.
- Ordering is FIFO per sender and receiver pair and nothing more. Every multi-hop protocol carries
  an epoch or correlation id and ignores stale replies. A worker result arriving after a cancel is
  dropped by the same rule.
- Determinism is the payoff of thread agnosticism. A sim host with one executor, a virtual clock,
  and scripted input runs the same actors as the terminal, window, and bare-metal hosts and asserts
  on the display grid. The ten second Hello World finishes in microseconds. Build that host before
  the TUI host once the FAA begins.
- Tracing: put a sequence number and the causing message's sequence in the header. A keystroke's
  whole causal tree then falls out of the log, which is what the management system idea wants, and
  tprobe gives per-hop latency.
- Hello World in this model is the app, label, display, timer, and supervisor actors plus one
  presenter thread. The trivial non-FAA version remains the right first step and later serves as
  the overhead baseline.

One host constraint to keep in view: window toolkits own the main thread on Mac, iOS, and Android.
There the main thread is the window device and presenter, the UI executor is its own thread, and
each keystroke crosses from the window thread to the executor and back. Real-time executors on
Linux also need rtprio limits or a capability before SCHED_FIFO takes effect.

# References

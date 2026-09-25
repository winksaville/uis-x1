# Architecture

The display path's current design: the layers from what the program shows to the screen, the two
shapes a program's loop takes, and the workspace conventions that follow. This file is revised
whenever the design moves. The discussion that set the direction is
[actor-model-1.md](actor-model-1.md), and [Terms in the actor notes](#terms-in-the-actor-notes)
maps its words onto these.

## Layers

Each layer knows only the one below it, so a layer can be swapped without the others noticing.

- Core: decides what to show and emits it as data, a cell grid first and a draw list beside it
  later. It is not built yet, and the text in each twin's renderer module stands in for it.
- Drawer: turns part of the core's output into pixels, a bitmap font, a shape, later outline text
  or a software 3D rasterizer. A drawer draws into a canvas and knows nothing of the screen.
- Canvas: the pixel surface drawers write into, `pixel-canvas`. It knows nothing of what is drawn.
- Frame: one complete picture, everything shown at one moment, drawn in full and then presented,
  the sense of frame rate and framebuffer. The window is where it is shown and the canvas where it
  is drawn. A renderer makes it and a presenter shows it, and each twin's `renderer` module draws
  today's, the background and the text.
  - The renderer draws into storage the presenter owns, softbuffer's buffer, the minifb host's,
    or a memory-mapped framebuffer, so a frame costs no allocation and no copy, and there is no
    frame type yet.
  - A frame type arrives with the actors, as the message a renderer sends a presenter: a pool
    slot of pixels or a grid, per [Rendering with actors](#rendering-with-actors).
- Presenter: puts a canvas's pixels on a screen, softbuffer or minifb in a window, a memory-mapped
  framebuffer later. It knows nothing of how the pixels were made.
- Renderer: the chain that turns the core's output into one kind of output, ANSI bytes for a
  terminal, drawers over a canvas for pixels, or later draw commands for a GPU.
- Host: a twin's binary. It owns or joins the loop, waits for input and time, and runs the core,
  the renderer, and the presenter in turn.
- Twin: a host program for one stack, named for that stack. The twins do the same thing, so they
  differ only in the stack under them.

## Pipelines

Every row starts from the core's output, a cell grid first and a draw list beside it later, and
each is one renderer and presenter pair, CPU and GPU alike. The frame column is what one complete
picture is in that row, what the renderer makes and the presenter shows.

| Output | Renderer | Frame | Presenter | Twin |
|---|---|---|---|---|
| Terminal | ANSI renderer, the grid to bytes | A screen of text, as bytes | The terminal | `tui-rustix` |
| Window | Drawers over `pixel-canvas` | The canvas's pixels | softbuffer | `gui-winit-softbuffer` |
| Window | Drawers over `pixel-canvas` | The canvas's pixels | minifb | `gui-minifb` |
| Bare-metal display | Drawers over `pixel-canvas` | The canvas's pixels | A memory-mapped framebuffer | Later |
| Window, GPU | GPU renderer, the output to draw commands | A swapchain image | The wgpu swapchain | Later |

- Today the text stands in for the grid. The terminal twin writes it as bytes, and the winit twin
  draws it with a bitmap font onto the canvas.
- Free 2D drawing, a curve or a pencil stroke, does not fit a cell grid, which is why the core's
  output gains a draw list, the same input a GPU renderer takes.

## The canvas

`pixel-canvas` is a surface and nothing more, so every pixel drawer shares it and every pixel
presenter shows it.

- It wraps a slice of `0x00RRGGBB` pixels, the form softbuffer and minifb present, at the buffer's
  own resolution, and clips at every edge.
- It implements the embedded-graphics draw-target trait, so fonts, shapes, and images from any
  crate built on that trait draw into it. It depends on `embedded-graphics-core` alone, the trait,
  the colors, and the geometry, and the drawers bring the fonts and shapes.
- `Scaled` wraps any draw target and draws each pixel as a square, for what should stay blocky at
  a higher resolution, the bitmap font for one. Scaling is a pixel operation, not a font's.
- `Ratio` carries a scale as an exact, reduced fraction, so a factor like 5/4 reaches the drawer
  unrounded. It is the scale factor's type, not geometry's.
- It will gain a retained layer that strokes accumulate in, and a blend of color by coverage for
  antialiased drawers, per [Generalize the pixel
  canvas](../TODO.md#generalize-the-pixel-canvas).
- Drawers stack above it without changing it. Text along a curved path, each character scaled, is
  layout, a transform per glyph, and an outline rasterizer, and only the last step touches the
  canvas, through the blend ([Draw text along a path](../TODO.md#draw-text-along-a-path)). A
  software 3D rasterizer would draw into the same surface.

## Two loop shapes

A host's loop takes one of two shapes, and the core is a non-blocking step that fits either:
drain its messages, update its state, emit its output, and return.

- Toolkit-owned: the window toolkit runs the loop and calls the host's handlers, as winit's
  application trait does in `gui-winit-softbuffer`. On macOS, iOS, and Android the toolkit also
  owns the main thread, so this is the shape the actor model must fit on those systems. The core
  runs as a step inside the handlers, or on its own thread and wakes the loop through winit's
  proxy.
- Program-owned: the host runs its own loop, polling input, stepping the core, drawing, and
  presenting, as `gui-minifb` does, and as the terminal twin's blocking read does in its
  simplest form. It is the bare-metal shape, where the loop waits for an interrupt and the
  interrupt handlers push into the rings.
- `gui-minifb` forgets its window on Wayland. minifb 0.28's Wayland backend destroys its event
  queue before the objects on it, and libwayland warns, so process exit releases the window
  instead, while on X11 it drops as usual, until the fix in [Fix minifb's Wayland drop order
  upstream](../TODO.md#fix-minifbs-wayland-drop-order-upstream) lands.

## The scale factor

A display's scale factor, 2 on a typical high-density screen and fractions like 1.25 in between,
is an input to drawing, and only a toolkit host reports it.

- Bitmap fonts come in fixed sizes, strikes, so a high-density display wants a larger strike.
  Doubling a single strike's pixels, the `Scaled` adapter, is the fallback for a font with one
  size, and only whole steps keep every font pixel an even, hard-edged square.
- Whole steps are a choice for sharpness and simplicity, not a limit of bitmap fonts. An area
  coverage resampler scales one by a fraction with even strokes and antialiased edges, and sharp
  bilinear, doubling by the whole part and smoothing only the rest, keeps the blur to about a
  pixel. Both need the canvas's blend.
- Outline fonts are curves, rasterized at any size, so a high-density display just asks for a
  larger one: the point size times the dots per inch over 72 times the scale factor. They stay
  sharp at fractional factors too.
- Either way the drawer needs the factor before it draws. Enlarging finished pixels afterwards is
  soft at fractional factors, whatever the font.
- The factor travels as a `Ratio`. Every platform's factor is a fraction, Wayland's over 120 and
  DPI over 96, so a ratio holds it exactly with no floating point, where an `f64` rounds some. A
  drawer decides what to do with it: bitmap text rounds it to a whole step, and a drawer that
  rotates or curves converts it once, at its transform.
- winit reports the factor, so `gui-winit-softbuffer` draws at the display's real resolution. It
  reads winit's factor as a ratio over 120, and its bitmap font rounds that to a whole step.
- minifb reports none. Its scale option is a window multiplier, not the display's factor, and on
  Wayland it hands the compositor a buffer at scale 1, which the compositor enlarges. So
  `gui-minifb` draws at scale 1 and the platform enlarges the result, softly at a fractional
  factor. On X11 nothing enlarges it.
- Bare metal matches minifb: nothing reports a factor, and the program, which knows its display,
  picks a strike or a scale when it is built.

## Rendering with actors

UI elements as actors, a button, a text input, a title, never draw. They report changes as data,
and one renderer draws the whole frame when one is due.

1. A widget actor owns its state, and on a change it sends a message describing its new look as
   data, its cells or its part of the draw list, never pixels.
2. One UI actor, the core or a compositor, holds the whole scene and marks what changed as dirty.
3. The host's frame clock asks for a frame: winit's redraw request, each pass of the minifb loop,
   a vertical blank interrupt or a timer on bare metal, and input or a timer on the terminal.
   Requests coalesce, so many ask for one frame.
4. On that tick, when anything is dirty, the renderer draws the scene once into a frame, a pool
   slot, and sends it to the presenter, which keeps only the latest.

- Why widgets do not draw on a change: the framebuffer would be mutable state shared by every
  widget actor, reachable other than by message. The presenter could show a half-updated frame,
  several changes between refreshes would draw several times, overlap would need agreement
  between actors, and a widget drawing pixels could not appear in the terminal.
- Why the request goes to the renderer, not each widget: one sender then paces the drawing, but
  widgets drawing on request still share the buffer and still draw pixels.
- What it buys: one draw per displayed frame, paced by the display, frames always complete when
  shown, widgets that render to ANSI or pixels alike, and dirty regions for the diffs a serial
  terminal wants and the damage rectangles softbuffer can present.
- Immediate mode is the simpler first step, as egui and Dear ImGui do it: on every tick the core
  rebuilds the whole scene from every widget's state, with no dirty tracking. It is cheap for a
  small UI and fits a cell grid, and dirty tracking comes when it is needed.

## GPU rendering

A GPU renderer is a sibling of the pixel path, not a layer under it or over it.

- The GPU draws the pixels itself from uploaded geometry, textures, and draw commands, and presents
  them through its own swapchain, so a GPU twin replaces both the drawers and the presenter, winit
  plus wgpu in place of winit plus softbuffer.
- It takes the same core output as the pixel path. Text on a GPU is usually a glyph atlas, or
  signed distance field glyphs where text must rotate and scale.
- Bare metal usually has no GPU driver, so the pixel path stays the one that is always available.

## Workspace conventions

- Twins are named for every layer of their stack that differs, `gui-winit-softbuffer` rather than
  `gui-softbuffer` once a twin without winit existed, and a twin's package and binary carry that
  name alone.
- Libraries are named for what they hold, `pixel-canvas` for a surface that renders nothing itself.
- The version-of-record is the workspace manifest's, and every member inherits it.
- Each twin keeps its own copy of the renderer, a few lines once the canvas is general, until the
  core supplies what it draws.
- Full validation installs every binary, so the installed twins always match the working copy.

## Terms in the actor notes

[actor-model-1.md](actor-model-1.md) was written before these layers were named, and its words map
onto them as follows.

- Its "ANSI presenter" and "pixel presenter" are renderers here, since they turn the grid into
  bytes or pixels. Its "pixel presenter" is the drawers over the canvas.
- Its "thin std hosts for terminal and window" are the hosts here, and what shows the pixels, which
  it leaves inside the window host, is the presenter here.
- Its "presenter thread" is a thread that runs a renderer and a presenter.

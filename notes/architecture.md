# Architecture

The display path's current design: the layers from what the program shows to the screen, the two
shapes a program's loop takes, and the workspace conventions that follow. This file is revised
whenever the design moves. The discussion that set the direction is
[actor-model-1.md](actor-model-1.md), and [Terms in the actor notes](#terms-in-the-actor-notes)
maps its words onto these.

## Layers

Each layer knows only the one below it, so a layer can be swapped without the others noticing.

- Core: decides what to show and emits it as data, a cell grid first and a draw list beside it
  later. It is not built yet, and each twin's own frame stands in for it.
- Drawer: turns part of the core's output into pixels, a bitmap font, a shape, later outline text
  or a software 3D rasterizer. A drawer draws into a canvas and knows nothing of the screen.
- Canvas: the pixel surface drawers write into, `pixel-canvas`. It knows nothing of what is drawn.
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
each is one renderer and presenter pair, CPU and GPU alike.

| Output | Renderer | Presenter | Twin |
|---|---|---|---|
| Terminal | ANSI renderer, the grid to bytes | The terminal | `tui-rustix` |
| Window | Drawers over `pixel-canvas` | softbuffer | `gui-winit-softbuffer` |
| Window | Drawers over `pixel-canvas` | minifb | `gui-minifb` |
| Bare-metal display | Drawers over `pixel-canvas` | A memory-mapped framebuffer | Later |
| Window, GPU | GPU renderer, the output to draw commands | The wgpu swapchain | Later |

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
  simplest form. It is the bare-metal shape, where the loop waits for an interrupt
  and the interrupt handlers push into the rings.

## The scale factor

A display's scale factor, 2 on a typical high-density screen and fractions like 1.25 in between,
is an input to drawing, and only a toolkit host reports it.

- Bitmap fonts come in fixed sizes, strikes, so a high-density display wants a larger strike.
  Doubling a single strike's pixels, the `Scaled` adapter, is the fallback for a font with one
  size, and only whole steps stay sharp.
- Outline fonts are curves, rasterized at any size, so a high-density display just asks for a
  larger one: the point size times the dots per inch over 72 times the scale factor. They stay
  sharp at fractional factors too.
- Either way the drawer needs the factor before it draws. Enlarging finished pixels afterwards is
  soft at fractional factors, whatever the font.
- winit reports the factor, so `gui-winit-softbuffer` draws at the display's real resolution and
  scales its bitmap font by the factor rounded to a whole step.
- minifb reports none. Its scale option is a window multiplier, not the display's factor, and on
  Wayland it hands the compositor a buffer at scale 1, which the compositor enlarges. So
  `gui-minifb` draws at scale 1 and the platform enlarges the result, softly at a fractional
  factor. On X11 nothing enlarges it.
- Bare metal matches minifb: nothing reports a factor, and the program, which knows its display,
  picks a strike or a scale when it is built.

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
- Each twin keeps its own copy of the frame, a few lines once the canvas is general, until the core
  supplies it.
- Full validation installs every binary, so the installed twins always match the working copy.

## Terms in the actor notes

[actor-model-1.md](actor-model-1.md) was written before these layers were named, and its words map
onto them as follows.

- Its "ANSI presenter" and "pixel presenter" are renderers here, since they turn the grid into
  bytes or pixels. Its "pixel presenter" is the drawers over the canvas.
- Its "thin std hosts for terminal and window" are the hosts here, and what shows the pixels, which
  it leaves inside the window host, is the presenter here.
- Its "presenter thread" is a thread that runs a renderer and a presenter.

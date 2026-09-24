# Notes

Design notes and records for the project, one file per topic. Conventions for the notes files,
`TODO.md` included, are in [agent-data/notes.md](../agent-data/notes.md), the one source of truth,
and this file points back at it.

- [actor-model-1.md](actor-model-1.md): the captured discussion that set the direction. TUI versus
  GUI under the final architecture's constraints, an actor runtime over the zc-ring rings and
  pools, UI elements as actors, and the message-level consequences.
- [architecture.md](architecture.md): the display path's current design. The layers, core,
  drawer, canvas, presenter, renderer, and host, the CPU and GPU pipelines, the two loop shapes,
  and the workspace conventions.
- [terminal-host.md](terminal-host.md): what the terminal host has to know about input and output
  modes. The keys normal mode keeps, what raw mode costs, the keys that never arrive, key press,
  repeat, and release, and what a Windows console needs.

#import "util.typ": *
#import "@preview/lilaq:0.6.0" as lq
#import "tuni-style.typ": *
#import "exec.typ": gantt

#let show-thr = true
#let x-units-in = "us"
#let x-units-out = "ms"

// ── Page setup ─────────────────────────────────────────────
#set page(margin: 0.25cm, width: auto, height: auto)
#set text(size: tuni-font-size)
#show lq.selector(lq.tick-label): set text(size: tuni-font-size-graph-min)
#show lq.selector(lq.diagram): set text(size: tuni-font-size-graph-min)

#gantt(
  "../ui-test/exec-mutex.csv",
  [Task Execution (Mutex)],
  x-units-in: x-units-in,
  x-units-out: x-units-out,
  ceiling-fpath: if show-thr { "../ui-test/ceil-mutex.csv" },
)

#gantt(
  "../ui-test/exec-rw.csv",
  [Task Execution (RW Lock)],
  x-units-in: x-units-in,
  x-units-out: x-units-out,
  ceiling-fpath: if show-thr { "../ui-test/ceil-rw.csv" },
)

#import "lib.typ": *
#import "@preview/lilaq:0.6.0" as lq

// ── Two alternating colours per task ───────────────────────
#let task-colors = (
  Writer: (color.hsl(210deg, 60%, 45%), color.hsl(210deg, 60%, 78%)),
  ReaderLow: (color.hsl(140deg, 55%, 35%), color.hsl(140deg, 55%, 70%)),
  J: (color.hsl(30deg, 80%, 45%), color.hsl(30deg, 80%, 78%)),
  ReaderHigh: (color.hsl(0deg, 65%, 45%), color.hsl(0deg, 65%, 78%)),
  Teardown: (color.hsl(280deg, 50%, 45%), color.hsl(280deg, 50%, 78%)),
)
#let fallback-palette = (
  (color.hsl(210deg, 60%, 45%), color.hsl(210deg, 60%, 78%)),
  (color.hsl(140deg, 55%, 35%), color.hsl(140deg, 55%, 70%)),
  (color.hsl(30deg, 80%, 45%), color.hsl(30deg, 80%, 78%)),
  (color.hsl(0deg, 65%, 45%), color.hsl(0deg, 65%, 78%)),
  (color.hsl(280deg, 50%, 45%), color.hsl(280deg, 50%, 78%)),
)

// ── Build a Gantt diagram from an exec-type CSV ────────────
#let parse-exec-csv(fpath) = {
  let raw = csv(fpath, row-type: array)

  let rows = raw.slice(1)
  let job-counter = (:)
  let exec-intervals = ()
  for r in rows {
    let task = r.at(2)
    let n = job-counter.at(task, default: 0)
    job-counter.insert(task, n + 1)
    exec-intervals.push((
      start: int(r.at(0)),
      end: int(r.at(1)),
      task: task,
      prio: int(r.at(3)),
      job: n,
    ))
  }
  exec-intervals
}

#let gantt(fpath, title, x-units-in: "cycles", x-units-out: "cycles") = {
  let exec-intervals = parse-exec-csv(fpath)

  // Collect unique tasks, sorted by priority ascending
  let seen = (:)
  let task-list = ()
  for iv in exec-intervals {
    if iv.task not in seen {
      seen.insert(iv.task, iv.prio)
      task-list.push((name: iv.task, prio: iv.prio))
    }
  }
  task-list = task-list.sorted(key: t => t.prio)

  let task-y = (:)
  for (i, t) in task-list.enumerate() {
    task-y.insert(t.name, i)
  }

  let convert-units = unit-conversions.at(x-units-in).at(x-units-out)

  // X-range shared across both panels
  let all-cycles = exec-intervals.map(iv => convert-units(iv.start)) + exec-intervals.map(iv => convert-units(iv.end))
  let x-min = calc.min(..all-cycles)
  let x-max = calc.max(..all-cycles)

  // Bars — colour by per-task job number from CSV
  let bar-h = 0.55
  let rects = exec-intervals.map(iv => {
    let y = task-y.at(iv.task)
    let pair = task-colors.at(
      iv.task,
      default: fallback-palette.at(calc.rem(iv.job, fallback-palette.len())),
    )
    let col = pair.at(calc.rem(iv.job, 2))
    lq.rect(
      convert-units(iv.start),
      y - bar-h / 2,
      width: convert-units(iv.end - iv.start),
      height: bar-h,
      fill: col,
      stroke: col.darken(15%) + 0.4pt,
      radius: 1pt,
    )
  })

  // Count jobs per task
  let job-count = (:)
  for iv in exec-intervals {
    let n = job-count.at(iv.task, default: 0)
    if iv.job + 1 > n { job-count.insert(iv.task, iv.job + 1) }
  }

  // Small vertical lines for theoretical arrival times
  let periods_ms = (0.5, 0.3, 0.2, 0.1)
  let vline-xs = periods_ms.map(period => {
    range(1, int(x-max / period) + 1).map(n => n * period)
  })

  lq.diagram(
    title: title,
    xlabel: x-units-out,
    width: 16cm,
    height: 3.5cm,
    xlim: (0, 3.0),
    ylim: (-0.7, task-list.len() - 0.3),
    yaxis: (
      ticks: task-list.enumerate().map(((i, t)) => (i, [#t.name])),
    ),
    xaxis: (position: bottom),
    lq.axis(
      kind: "y",
      position: right,
      ticks: task-list
        .enumerate()
        .map(((i, t)) => {
          let n = job-count.at(t.name, default: 0)
          (i, [#n jobs])
        }),
    ),
    ..range(0, 4)
      .map(idx => {
        (
          lq.vlines(
            ..vline-xs.at(idx).enumerate().filter(it => calc.even(it.at(0))).map(it => it.at(1)),
            min: idx + 0.2,
            max: idx + 0.4,
            stroke: 0.4pt + task-colors.values().at(idx).at(0),
          ),
          lq.vlines(
            ..vline-xs.at(idx).enumerate().filter(it => calc.odd(it.at(0))).map(it => it.at(1)),
            min: idx + 0.2,
            max: idx + 0.4,
            stroke: stroke(0.4pt + task-colors.values().at(idx).at(1)),
          ),
        )
      })
      .flatten(),
    ..rects,
  )
}

// ── Page setup ─────────────────────────────────────────────
#set page(margin: 0.5cm, width: auto, height: auto)
#set text(size: 10pt)

#gantt("../ui-test/exec-mutex.csv", [Mutex — Task Execution], x-units-in: "us", x-units-out: "ms")

#gantt("../ui-test/exec-rw.csv", [RW Lock — Task Execution], x-units-in: "us", x-units-out: "ms")

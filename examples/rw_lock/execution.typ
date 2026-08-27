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

// ── Build a Gantt diagram from an exec CSV ────────────
#let gantt(csv-path, title) = {
  let raw = csv(csv-path, row-type: array)

  let exec-intervals = raw
    .slice(1)
    .map(r => (
      start: int(r.at(0)),
      end: int(r.at(1)),
      task: r.at(2),
      prio: int(r.at(3)),
      job: int(r.at(4)),
    ))

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

  // X-range shared across both panels
  let all-cycles = exec-intervals.map(iv => iv.start) + exec-intervals.map(iv => iv.end)
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
      iv.start,
      y - bar-h / 2,
      width: iv.end - iv.start,
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

  lq.diagram(
    title: title,
    xlabel: [us],
    width: 16cm,
    height: 3.5cm,
    xlim: (x-min, x-max),
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
    ..rects,
  )
}

// ── Page setup ─────────────────────────────────────────────
#set page(margin: 0.5cm, width: auto, height: auto)
#set text(size: 10pt)

#gantt("ui-test/exec-mutex.csv", [Mutex — Task Execution])

//#gantt("ui-test/exec-rw.csv", [RW Lock — Task Execution])

#import "lib.typ": *
#import "@preview/lilaq:0.6.0" as lq

// Configurations, keep aligned with main.rs
#let pre-trigger = true
#let periods_ms = (0.5, 0.3, 0.2, 0.1)
#let arrival_offset_us = 10

// ── Two alternating colours per task ───────────────────────
#let task-colors = (
  Writer: (color.hsl(200deg, 60%, 45%), color.hsl(220deg, 60%, 78%)),
  ReaderLow: (color.hsl(130deg, 55%, 35%), color.hsl(150deg, 55%, 70%)),
  J: (color.hsl(20deg, 80%, 45%), color.hsl(40deg, 80%, 78%)),
  ReaderHigh: (color.hsl(-10deg, 65%, 45%), color.hsl(10deg, 65%, 78%)),
  Teardown: (color.hsl(270deg, 50%, 45%), color.hsl(290deg, 50%, 78%)),
)

/* Parses intervals from an execution trace format CSV, like so:
 *
 * ```
 * (
 *   (
 *     start: 11,
 *     end: 26,
 *     task: "ReaderHigh",
 *     prio: 252,
 *     job: 0,
 *   ),
 *   ...
 * ),
 */
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

/* Returns tasks by priority, like so:
 *
 * ```
 * (
 *   (name: "Writer", prio: 248),
 *   (name: "ReaderLow", prio: 249),
 *   (name: "J", prio: 251),
 *   (name: "ReaderHigh", prio: 252),
 * )
 * ```
 */
#let tasks-by-priority(intervals) = {
  let seen = (:)
  let tasks = ()
  for interval in intervals {
    if interval.task not in seen {
      seen.insert(interval.task, interval.prio)
      tasks.push((name: interval.task, prio: interval.prio))
    }
  }
  tasks.sorted(key: task => task.prio)
}

#let task-color(interval) = {
  task-colors.at(interval.task).at(calc.rem(interval.job, 2))
}

#let subtract-overlap(parts, blocker) = {
  let remaining = ()
  for part in parts {
    if blocker.end <= part.start or blocker.start >= part.end {
      remaining.push(part)
    } else {
      if part.start < blocker.start {
        remaining.push((start: part.start, end: blocker.start))
      }
      if blocker.end < part.end {
        remaining.push((start: blocker.end, end: part.end))
      }
    }
  }
  remaining
}

#let active-parts(interval, intervals) = {
  let parts = ((start: interval.start, end: interval.end),)
  for blocker in intervals {
    if blocker.prio > interval.prio {
      parts = subtract-overlap(parts, blocker)
    }
  }
  parts.filter(part => part.end - part.start > 1)
}

#let preempted-parts(interval, intervals) = {
  let parts = ((start: interval.start, end: interval.end),)
  for active in active-parts(interval, intervals) {
    parts = subtract-overlap(parts, active)
  }
  parts.filter(part => part.end > part.start)
}

#let draw-rects(items, convert-units, height) = {
  items.map(item => {
    let interval = item.interval
    let y = item.y
    let fill = item.fill
    lq.rect(
      convert-units(interval.start),
      y - height / 2,
      width: convert-units(interval.end - interval.start),
      height: height,
      fill: fill,
    )
  })
}

// ── Build a Gantt diagram from an exec-type CSV ────────────
#let gantt(fpath, title, x-units-in: "cycles", x-units-out: "cycles") = {
  let exec-intervals = parse-exec-csv(fpath)
  let task-list = tasks-by-priority(exec-intervals)

  let task-y = (:)
  for (index, task) in task-list.enumerate() {
    task-y.insert(task.name, index)
  }

  let convert-units = unit-conversions.at(x-units-in).at(x-units-out)
  let all-cycles = (
    exec-intervals.map(interval => convert-units(interval.start))
      + exec-intervals.map(interval => convert-units(interval.end))
  )
  let x-max = calc.max(..all-cycles)

  // Draw muted preempted pieces below the solid active pieces.
  let preempted-bar-h = 0.25
  let active-bar-h = 0.55
  let preempted-items = exec-intervals
    .map(interval => {
      let y = task-y.at(interval.task)
      let fill = color.mix(task-color(interval), luma(65%)).desaturate(60%)
      preempted-parts(interval, exec-intervals).map(part => (
        interval: part,
        y: y,
        fill: fill,
      ))
    })
    .flatten()
  let preempted-rects = draw-rects(preempted-items, convert-units, preempted-bar-h)

  let active-items = exec-intervals
    .map(interval => {
      let y = task-y.at(interval.task)
      active-parts(interval, exec-intervals).map(part => (
        interval: part,
        y: y,
        fill: task-color(interval),
      ))
    })
    .flatten()
  let active-rects = draw-rects(active-items, convert-units, active-bar-h)

  // Count jobs per task for the right-hand axis.
  let job-count = (:)
  for interval in exec-intervals {
    let count = job-count.at(interval.task, default: 0)
    if interval.job + 1 > count {
      job-count.insert(interval.task, interval.job + 1)
    }
  }

  // Small vertical lines for theoretical arrival times.
  let vline-xs = periods_ms.map(period => {
    range(1, int(x-max / period) + 1 + if pre-trigger { 1 }).map(n => (
      (n - if pre-trigger { 1 } else { 0 }) * period + arrival_offset_us / 1000
    ))
  })

  let arrival-lines = range(0, 4)
    .map(index => (
      lq.vlines(
        ..vline-xs.at(index).enumerate().filter(it => calc.even(it.at(0))).map(it => it.at(1)),
        min: index + 0.2,
        max: index + 0.4,
        stroke: 0.6pt + task-colors.values().at(index).at(0),
      ),
      lq.vlines(
        ..vline-xs.at(index).enumerate().filter(it => calc.odd(it.at(0))).map(it => it.at(1)),
        min: index + 0.2,
        max: index + 0.4,
        stroke: stroke(0.6pt + task-colors.values().at(index).at(1)),
      ),
    ))
    .flatten()

  lq.diagram(
    title: title,
    xlabel: x-units-out,
    width: 16cm,
    height: 3.5cm,
    xlim: (0, 3.0),
    ylim: (-0.7, task-list.len() - 0.3),
    yaxis: (
      ticks: task-list.enumerate().map(((index, task)) => (index, [#task.name])),
    ),
    xaxis: (position: bottom),
    lq.axis(
      kind: "y",
      position: right,
      ticks: task-list
        .enumerate()
        .map(((index, task)) => {
          let count = job-count.at(task.name, default: 0)
          (index, [#count jobs])
        }),
    ),
    ..arrival-lines,
    ..preempted-rects,
    ..active-rects,
  )
}

// ── Page setup ─────────────────────────────────────────────
#set page(margin: 0.5cm, width: auto, height: auto)
#set text(size: 10pt)

#gantt("../ui-test/exec-mutex.csv", [Mutex — Task Execution], x-units-in: "us", x-units-out: "ms")

#gantt("../ui-test/exec-rw.csv", [RW Lock — Task Execution], x-units-in: "us", x-units-out: "ms")

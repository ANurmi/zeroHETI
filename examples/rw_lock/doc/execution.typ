#import "lib.typ": *
#import "@preview/lilaq:0.6.0" as lq
#import "tuni-style.typ": *

// Data configurations, keep aligned with main.rs
#let pre-trigger = true
#let periods_ms = (0.5, 0.3, 0.2, 0.1)
#let arrival_offset_us = 10
#let arrival_offset_ms = arrival_offset_us / 1000
// What part of data should be shown on X-axis?
#let xlim = (arrival_offset_ms, 3 + arrival_offset_ms)
#let x-units-in = "us"
#let x-units-out = "ms"

// # Styles
#let show-thr = true
#let show-color = true
#let styles = (
  diagram-width: 16cm,
  diagram-height: 3.5cm,
  arrival-tick-thickness: 0.6pt,
  arrival-tick-pos: 0.3,
  arrival-tick-h: 0.1,
  // Height of the active bar sections
  active-bar-h: 0.55,
  // Height of the preempted bar sections
  preempted-bar-h: 0.25,
  task-color-base: {
    let comp(c) = if show-color { c.components() } else { (0, 0, c.components().at(2)) }
    (
      oklch(40%, comp(tuni-purple).at(1), comp(tuni-purple).at(2)),
      oklch(40%, comp(tuni-pink).at(1), comp(tuni-pink).at(2)),
      oklch(40%, comp(tuni-blue).at(1), comp(tuni-blue).at(2)),
      oklch(40%, comp(tuni-fuchsia).at(1), comp(tuni-fuchsia).at(2)),
    )
  },
)
#let fn-revalue-parity(parity-idx) = {
  let dist = 30%
  c => {
    let l = c.components().at(0)
    if calc.rem(parity-idx, 2) == 1 {
      // Odd
      c.lighten(dist)
    } else {
      // Even
      c.darken(dist)
    }
  }
}
#let fn-lighten-inactive = {
  let dist = 60%
  c => c.lighten(dist)
}

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
  for iv in intervals {
    if iv.task not in seen {
      seen.insert(iv.task, iv.prio)
      tasks.push((name: iv.task, prio: iv.prio))
    }
  }
  tasks.sorted(key: task => task.prio)
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

#let active-parts(iv, intervals) = {
  let parts = ((start: iv.start, end: iv.end),)
  for blocker in intervals {
    if blocker.prio > iv.prio {
      parts = subtract-overlap(parts, blocker)
    }
  }
  parts.filter(part => part.end - part.start > 1)
}

#let preempted-parts(iv, intervals) = {
  let parts = ((start: iv.start, end: iv.end),)
  for active in active-parts(iv, intervals) {
    parts = subtract-overlap(parts, active)
  }
  parts.filter(part => part.end > part.start)
}

#let draw-rects(items, convert-units, height) = {
  items.map(item => {
    let iv = item.iv
    lq.rect(
      convert-units(iv.start),
      item.y - height / 2,
      width: convert-units(iv.end - iv.start),
      height: height,
      fill: item.fill,
    )
  })
}

#let gen-minilines(xs, ts-max, ypos, line-count) = {
  range(0, line-count)
    .map(index => {
      let y-min = index + ypos - styles.arrival-tick-h
      let y-max = index + ypos + styles.arrival-tick-h
      let (even-colors, odd-colors) = (
        fn-revalue-parity(0)(styles.task-color-base.at(index)),
        fn-revalue-parity(1)(styles.task-color-base.at(index)),
      )
      (
        // Draw every line in one color, and every other in the other
        lq.vlines(
          ..xs.at(index).enumerate().filter(it => calc.even(it.at(0))).map(it => it.at(1)),
          min: y-min,
          max: y-max,
          stroke: styles.arrival-tick-thickness + even-colors,
        ),
        lq.vlines(
          ..xs.at(index).enumerate().filter(it => calc.odd(it.at(0))).map(it => it.at(1)),
          min: y-min,
          max: y-max,
          stroke: styles.arrival-tick-thickness + odd-colors,
        ),
      )
    })
    .flatten()
}


// ── Build a Gantt diagram from an exec-type CSV ────────────
#let ceiling-overlay(fpath, task-list, x-units-in: "cycles", x-units-out: "cycles") = {
  let (cycles, ceilings) = parse-ceil-csv(fpath)
  let zero-row = -1
  let prio-to-y = (:)
  for (index, task) in task-list.enumerate() {
    prio-to-y.insert(str(task.prio), index)
  }
  prio-to-y.insert("0", zero-row)
  let fn-convert-units = unit-conversions.at(x-units-in).at(x-units-out)

  let xs = ()
  let ys = ()
  for (index, ceiling) in ceilings.enumerate() {
    let threshold = if ceiling == "" { 0 } else { int(ceiling) }
    let y = prio-to-y.at(str(threshold), default: none)
    if y != none {
      xs.push(fn-convert-units(cycles.at(index)))
      ys.push(y)
    }
  }

  if xs.len() > 0 {
    lq.plot(
      xs,
      ys,
      step: end,
      stroke: tuni-black + 0.2pt,
      mark: none,
    )
  }
}

#let gantt(fpath, title, x-units-in: "cycles", x-units-out: "cycles", ceiling-fpath: none) = {
  // Intervals: `( (start: 11, end: 26, task: "ReaderHigh", prio: 252, job: 0, ), ..., )`
  let ivs = parse-exec-csv(fpath)
  // Task list: `( (name: "Writer", prio: 248), ..., )`
  let task-list = tasks-by-priority(ivs)

  // Draw muted preempted pieces below the solid active pieces.
  let task-ys = task-list.enumerate().map(((index, task)) => (task.name, index)).to-dict()
  let preempted-items = ivs
    .map(iv => {
      let y = task-ys.at(iv.task)
      preempted-parts(iv, ivs).map(part => (
        iv: part,
        y: y,
        fill: fn-lighten-inactive(fn-revalue-parity(iv.job)(styles.task-color-base.at(y))),
      ))
    })
    .flatten()
  let fn-convert-units = unit-conversions.at(x-units-in).at(x-units-out)
  let preempted-rects = draw-rects(preempted-items, fn-convert-units, styles.preempted-bar-h)

  let active-items = ivs
    .map(iv => {
      let y = task-ys.at(iv.task)
      active-parts(iv, ivs).map(part => (
        iv: part,
        y: y,
        fill: fn-revalue-parity(iv.job)(styles.task-color-base.at(y)),
      ))
    })
    .flatten()
  let active-rects = draw-rects(active-items, fn-convert-units, styles.active-bar-h)

  // Count jobs per task for the right-hand axis.
  let task-to-job-count = (:)
  for iv in ivs {
    let count = task-to-job-count.at(iv.task, default: 0)
    if iv.job + 1 > count {
      task-to-job-count.insert(iv.task, iv.job + 1)
    }
  }

  let ceiling-plot = if ceiling-fpath != none {
    ceiling-overlay(ceiling-fpath, task-list, x-units-in: x-units-in, x-units-out: x-units-out)
  } else { none }

  // Small vertical lines for theoretical arrival times
  let timestamps-merged = (
    ivs.map(iv => fn-convert-units(iv.start)) + ivs.map(iv => fn-convert-units(iv.end))
  )
  // Maximum timestamp in range
  let ts-max = calc.max(..timestamps-merged)
  let task-count = task-list.len()
  let arrival-xs = periods_ms.map(period => {
    range(1, int(ts-max / period) + 1 + if pre-trigger { 1 }).map(n => (
      (n - if pre-trigger { 1 } else { 0 }) * period + arrival_offset_ms
    ))
  })
  let arrival-lines = gen-minilines(arrival-xs, ts-max, styles.arrival-tick-pos, task-count)
  /*
  let dl-xs = periods_ms.map(period => {
    range(1, int(ts-max / period) + 1 + if pre-trigger { 1 }).map(n => (
      (n - if pre-trigger { 1 } else { 0 }) * period + offset + period
    ))
  })
  let dl-lines = gen-minilines(dl-xs, ts-max, -styles.arrival-tick-pos, task-count)
  */

  let right-axis-ticks = task-list
    .enumerate()
    .map(((index, task)) => {
      let count = task-to-job-count.at(task.name, default: 0)
      (
        index,
        grid(
          row-gutter: if ceiling-fpath != none { 0.15em } else { 0em },
          [#count jobs],
          if ceiling-fpath != none [π = #task.prio],
        ),
      )
    })
  if ceiling-fpath != none {
    let zero-row = -1
    right-axis-ticks.push((zero-row, [0]))
  }
  show: lq.set-diagram(
    xaxis: (
      position: bottom,
      ticks: range(0, 31).map(i => i * 0.2 + arrival_offset_ms),
      format-ticks: lq.tick-format.linear,
      subticks: 0,
      // Align x-axis with the first arrival for clarity
      offset: arrival_offset_ms,
    ),
    yaxis: (
      ticks: task-list.enumerate().map(((index, task)) => (index, [#task.name])),
    ),
  )
  lq.diagram(
    title: title,
    xlabel: x-units-out,
    width: styles.diagram-width,
    height: styles.diagram-height,
    xlim: xlim,
    ylim: (-0.7 - if ceiling-fpath != none { 0.7 } else { 0 }, task-list.len() - 0.3),
    lq.axis(
      kind: "y",
      position: right,
      ticks: right-axis-ticks,
    ),
    ..arrival-lines,
    //..dl-lines,
    ..preempted-rects,
    ..active-rects,
    ceiling-plot,
  )
}

// ── Page setup ─────────────────────────────────────────────
#set page(margin: 0.25cm, width: auto, height: auto)
#set text(size: tuni-font-size)
#show lq.selector(lq.tick-label): set text(size: tuni-font-size-graph-min)
#show lq.selector(lq.diagram): set text(size: tuni-font-size-graph-min)

#gantt(
  "../ui-test/exec-mutex.csv",
  [Mutex — Task Execution],
  x-units-in: x-units-in,
  x-units-out: x-units-out,
  ceiling-fpath: if show-thr { "../ui-test/ceil-mutex.csv" },
)

#gantt(
  "../ui-test/exec-rw.csv",
  [RW Lock — Task Execution],
  x-units-in: x-units-in,
  x-units-out: x-units-out,
  ceiling-fpath: if show-thr { "../ui-test/ceil-rw.csv" },
)

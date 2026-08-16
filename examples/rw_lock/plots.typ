#import "@preview/lilaq:0.6.0" as lq

#set page(width: auto, height: auto, margin: 2cm)

// ---- task colors ----
#let c_rh = blue
#let c_j = green
#let c_rl = orange
#let c_w = teal
#let c_miss = red

// ---- load per-variant job CSV (ui-test/mutex.csv, ui-test/rw.csv) ----
#let load(path) = lq.load-txt(read(path), header: true, converters: (
  mode: str,
  task: str,
  idx: int,
  resp_us: int,
  rel_us: int,
  miss: int,
  rest: int,
))
#let mutex = load("ui-test/mutex.csv")
#let rw = load("ui-test/rw.csv")

/// Extract (rel, resp, miss) arrays for one task from a loaded CSV dict.
#let jobs(d, task) = {
  let rel = ()
  let resp = ()
  let miss = ()
  for i in range(d.task.len()) {
    if d.task.at(i) == task {
      rel.push(int(d.rel_us.at(i)))
      resp.push(int(d.resp_us.at(i)))
      miss.push(int(d.miss.at(i)))
    }
  }
  (rel: rel, resp: resp, miss: miss)
}

// ============================================================
// Figure 1: J response time vs. release time, both variants.
// ============================================================
#let jm = jobs(mutex, "J")
#let jr = jobs(rw, "J")

#figure({
  show: lq.set-diagram(width: 9cm, height: 5cm)
  lq.diagram(
    title: [J response time vs. release time],
    xlabel: [time (µs)],
    ylabel: [response (µs)],
    xlim: (0, 20000),
    ylim: (0, 1200),
    lq.hlines(400, stroke: (paint: red, dash: "dashed"), label: [J deadline]),
    lq.scatter(jm.rel, jm.resp,
      mark: "x",
      color: jm.miss.map(m => if m == 1 { c_miss } else { c_rh }),
      label: [mutex]),
    lq.scatter(jr.rel, jr.resp,
      mark: "o",
      color: c_j,
      label: [rw-lock]),
  )
}, caption: [
  J response time (release to completion) for both variants. Red crosses are
  mutex jobs exceeding the #raw("400") µs relative deadline; the mutex
  scheduler never admits them on time, while under the rw-lock the response is
  below #raw("70") µs for every job.
])

// ============================================================
// Figure 2: Gantt timeline of all jobs, one panel per variant.
// ============================================================
#let task_y = (W: 0.0, RL: 1.0, J: 2.0, RH: 3.0)
#let task_fill = (RH: c_rh, J: c_j, RL: c_rl, W: c_w)

#let gantt(d, title) = {
  let bars = ()
  for i in range(d.task.len()) {
    let t = d.task.at(i)
    let miss = int(d.miss.at(i))
    bars.push(lq.rect(
      int(d.rel_us.at(i)),
      task_y.at(t),
      width: int(d.resp_us.at(i)),
      height: 0.7,
      fill: if t == "J" and miss == 1 { c_miss } else { task_fill.at(t) },
      align: left + horizon,
    ))
  }
  lq.diagram(
    title: title,
    xlabel: [time (µs)],
    xlim: (0, 20000),
    ylim: (-0.5, 3.5),
    yaxis: (
      ticks: (
        (0.5, [Writer]),
        (1.5, [ReaderLow]),
        (2.5, [J]),
        (3.5, [ReaderHigh]),
      ),
      subticks: none,
    ),
    ..bars,
  )
}

#let lentry(fill, name) = box(
  stroke: 0.5pt + gray,
  fill: fill,
  width: 0.8em,
  height: 0.8em,
) + h(0.3em) + name

#figure({
  show: lq.layout
  show: lq.set-diagram(width: 7cm, height: 3.6cm)
  grid(
    columns: 2,
    column-gutter: 1.2em,
    gantt(mutex, [mutex]),
    gantt(rw, [rw-lock]),
  )
  v(0.6em)
  lentry(c_rh, [ReaderHigh])
  h(1em)
  lentry(c_j, [J])
  h(1em)
  lentry(c_rl, [ReaderLow])
  h(1em)
  lentry(c_w, [Writer])
  h(1em)
  lentry(c_miss, [J deadline miss])
}, caption: [
  Job timeline: each bar spans release to completion at its release time
  (x) with length equal to its response time. Under the mutex, ReaderLow's
  long read critical section blocks ReaderHigh and J continuously, so many J
  jobs run late (red); under the rw-lock only the Writer is blocked by the
  same critical section, and all jobs meet their deadlines.
])

#import "@preview/lilaq:0.6.0" as lq

#let load-ceiling(csv-path) = {
  let data = csv(csv-path, row-type: array)
  let cycles = data.slice(1).map(row => int(row.at(0)))
  let ceilings = data
    .slice(1)
    .map(row => {
      let v = row.at(5)
      if v == "" { 0 } else { int(v) }
    })
  (cycles, ceilings)
}

#let ceiling-diagram(csv-path, title) = {
  let (cycles, ceilings) = load-ceiling(csv-path)
  lq.diagram(
    title: title,
    xlabel: [Cycle],
    ylabel: [Ceiling],
    width: 15cm,
    height: 5cm,
    ylim: (250, 253),
    lq.plot(
      cycles,
      ceilings,
      step: end,
      stroke: blue + 1.5pt,
      mark: none,
      label: [ceiling],
    ),
  )
}

#set page(margin: 0.5cm, width: auto, height: auto)
#set text(size: 10pt)

#let periods = (
  "ReaderHigh": 700,
  "J": 1300,
  "ReaderLow": 1000,
  "W": 1500,
)

#ceiling-diagram("ui-test/ceil-mutex.csv", [Resource Ceiling Evolution (Mutex)])

#ceiling-diagram("ui-test/ceil-rw.csv", [Resource Ceiling Evolution (RW)])

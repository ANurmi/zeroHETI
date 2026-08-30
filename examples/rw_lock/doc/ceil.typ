
#let parse-ceil-csv(fpath) = {
  let data = csv(fpath, row-type: array)
  let cycles = data.slice(1).map(row => int(row.at(0)))
  let ceilings = data
    .slice(1)
    .map(row => {
      let t = row.at(1)
      // Capture only changes in resource ceiling
      if t in ("acq", "rel") {
        let ceil = row.at(4)
        int(ceil)
      } else {
        // Task activation and completion also changes the effective ceiling,
        // but we ignore that in the graph.
        /*if t == "act" {
          let prio = row.at(3)
          int(prio)
        } else if t == "comp" {
          let stacked = row.at(5)
          int(stacked)
        } else */
        none
      }
    })
  let both = cycles.zip(ceilings).filter(((cc, ceil)) => { ceil != none })
  import "@preview/funarray:0.4.0"
  funarray.unzip(both)
}

#let ceiling-diagram(fpath, title, x-units-in: "cycles", x-units-out: "cycles") = {
  let (cycles, ceilings) = parse-ceil-csv(fpath)

  let convert-units = unit-conversions.at(x-units-in).at(x-units-out)

  let xdata = cycles.map(convert-units)
  let x-min = calc.min(..xdata)
  let x-max = calc.max(..xdata)

  lq.diagram(
    title: title,
    xlabel: x-units-out,
    ylabel: [Ceiling],
    width: 16cm,
    height: 3.5cm,
    xlim: (0, x-max),
    ylim: (246, 254),
    lq.plot(
      xdata,
      ceilings,
      stroke: blue + 1.5pt,
      mark: none,
    ),
  )
}

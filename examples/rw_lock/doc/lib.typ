#import "@preview/lilaq:0.6.0" as lq

#let unit-conversions = (
  "us": ("ms": x => x / 1000),
)

#let parse-ceil-csv(fpath) = {
  let data = csv(fpath, row-type: array)
  let cycles = data.slice(1).map(row => int(row.at(0)))
  let ceilings = data
    .slice(1)
    .map(row => {
      let v = row.at(4)
      if v == "" { 0 } else { int(v) }
    })
  (cycles, ceilings)
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

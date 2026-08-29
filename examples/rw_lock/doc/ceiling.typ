#import "lib.typ": *
#import "@preview/lilaq:0.6.0" as lq

#set page(margin: 0.5cm, width: auto, height: auto)
#set text(size: 10pt)

#let periods = (
  "ReaderHigh": 700,
  "J": 1300,
  "ReaderLow": 1000,
  "W": 1500,
)

#ceiling-diagram("../ui-test/ceil-mutex.csv", [Resource Ceiling Evolution (Mutex)], x-units-in: "us", x-units-out: "ms")

#ceiling-diagram("../ui-test/ceil-rw.csv", [Resource Ceiling Evolution (RW)], x-units-in: "us", x-units-out: "ms")

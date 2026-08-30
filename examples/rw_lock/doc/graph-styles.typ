#import "tuni-style.typ": *

// Data configurations, keep aligned with main.rs
#let pre-trigger = true
#let periods_ms = (0.5, 0.3, 0.2, 0.1)
#let dl_perios_ms = (none, none, 0.1, none)
#let arrival_offset_us = 10
#let arrival_offset_ms = arrival_offset_us / 1000
// What part of data should be shown on X-axis?
#let xlim = (arrival_offset_ms, 3 + arrival_offset_ms)

// # Styles
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
  ceiling-stroke: tuni-grey + 0.5pt,
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

# rw_lock — readers-writer lock schedulability demo

Cycle-accurate (Verilator, 10 MHz) demonstration on zeroHETI/RTIC that replacing
a **mutex** on a shared resource with a **readers-writer lock** lowers the
worst-case response time of an independent high-priority job `J`, turning an
unschedulable system schedulable. The task set is engineered to satisfy
Theorem 3.9's boundary conditions C1–C5 (see `2026-08-07-abstract-rw-demo-plan.md`
at the repo root), the regime the drone controller (`../rt_prof`) does *not*
exercise.

A single binary covers both runs; a feature flag toggles the readers' lock type.

## Task set (C1–C5 mapping)

| Task       | IRQ          | Priority π | Access to `R` | Role                                   |
|------------|--------------|------------|---------------|----------------------------------------|
| ReaderHigh | `Timer0Cmp`  | `0xFC`     | read          | highest-π accessor → **C2**            |
| J          | `Timer1Cmp`  | `0xFB`     | none          | measured job → **C1**, **C3**          |
| ReaderLow  | `Timer2Cmp`  | `0xF9`     | read, **long** CS | **C4/C5**: `B(J)` = this CS        |
| Writer     | `Timer3Cmp`  | `0xF8`     | write (short) | → **C3**                               |
| Control    | `MachineTimer`| `0xFF`    | none          | setup / teardown + verdict             |

- Mutex ceiling `⌈R⌉0 = 0xFC`; read ceiling `⌈R⌉1 = 0xF8` (max priority among
  writers). Both are computed by the RTIC macro and printed at build time
  (`[RTIC] * r @π=252`).
- **Mutex build**: ReaderLow's `lock()` raises the system ceiling (`mintthresh`)
  to `0xFC ≥ π(J)`, so `J` is blocked for the whole read critical section.
- **RW build**: ReaderLow uses `read_lock()`, which only raises the ceiling to
  `0xF8 < π(J)`, so `J` preempts the read critical section.

Periods (µs): ReaderHigh 700, J 1300, ReaderLow 1000, Writer 1500. `J`'s deadline
is 400 µs, chosen between `WCRT(J)_mutex` and `WCRT(J)_rw`.

## Measurement

Each task reads its own APB timer counter at exec entry/exit. The counter resets
to zero at the compare (= release), so the exit value is the response time
(single-wrap guard). Statistics are recorded per task and printed by the
`Control` teardown along with `J`'s deadline-miss count and a verdict.

## How to run

Requires the toolchain from `source .env` at the repo root (prebuilt Verilator
binary present; no rebuild needed).

```sh
# Mutex build (J must be unschedulable)
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic

# RW-lock build (J must be schedulable)
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic,rw
```

Optional compile-time knobs (env vars):

- `RUNTIME_MS` — runtime in ms (required)
- `RL_CS_ITERS` — ReaderLow critical-section length in busy-loop iterations (default 900)
- `DL_J_US` — J's deadline in µs (default 400)

Alternatively use `just`:

```sh
just run-mutex
just run-rw
just compare      # runs both and prints a compact side-by-side summary
```

## Expected results (representative, 20 ms runs)

|                        | Mutex                | RW-lock              |
|------------------------|----------------------|----------------------|
| ReaderHigh worst (µs)  | ~680                 | ~29                  |
| J worst (µs)           | ~880                 | ~70                  |
| J deadline misses      | 6–8 / ~14            | 0 / 15               |
| ReaderLow worst (µs)   | ~1020                | ~1020                |
| Writer worst (µs)      | ~1490                | ~1475                |
| Verdict                | J NOT schedulable    | J schedulable        |

`Δ ≈ WCRT(J)_mutex − WCRT(J)_rw ≈ |ReaderLow read CS|`, matching Lemma 2.17 /
Theorem 3.9. The residual (J worst ~70 µs in RW mode) is `J`'s own execution
time plus ReaderHigh preemption.

Note: run-to-run variance of a few jobs is expected (the worst-case overlap of
`J`'s releases with ReaderLow's critical section is what produces the misses in
the mutex build). The qualitative result — `J` misses in the mutex build, meets
its deadline in the RW build — is stable.

## Implementation notes

The RTIC fork's parser has no `cfg` support on `#[task]` items, so each reader is
declared twice (struct + impl), gated by `#[cfg(feature = "rw")]` /
`#[cfg(not(feature = "rw"))]`. The fork's parser was extended to evaluate these
predicates against the env var `RTIC_CFG_FEATURE_<NAME>` (forwarded from cargo by
`build.rs`; `CARGO_FEATURE_*` is not visible to proc macros). Inactive variants
are emitted as stripped `other_code` and discarded by `rustc`.

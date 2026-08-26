# rw_lock — readers-writer lock schedulability demo

Cycle-accurate (Verilator, 100 MHz) demonstration on zeroHETI/RTIC that replacing
a **mutex** on a shared resource with a **readers-writer lock** lowers the
worst-case response time of an independent high-priority job `J`, turning an
unschedulable system schedulable. The task set is engineered to satisfy
Theorem 3.9's boundary conditions C1–C5.

A single source covers both cases; a feature flag toggles the readers' lock type.

```sh
# Run with mutex
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic

# Run with RW locks
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic,rw
```

## Task set (C1–C5 mapping)

| Task       | IRQ            | Priority π     | Access to `R`     | Role                          |
| ---------- | -------------- | -------------- | ----------------- | ----------------------------- |
| ReaderHigh | `Timer0Cmp`    | `0xFC` / `252` | read              | highest-π accessor → **C2**   |
| J          | `Timer1Cmp`    | `0xFB` / `251` | none              | measured job → **C1**, **C3** |
| ReaderLow  | `Timer2Cmp`    | `0xF9` / `249` | read, **long** CS | **C4/C5**: `B(J)` = this CS   |
| Writer     | `Timer3Cmp`    | `0xF8` / `248` | write (short)     | → **C3**                      |
| (Teardown) | `MachineTimer` | `0xFF` / `255` | none              | setup / teardown + verdict    |

- Mutex ceiling $⌈R⌉_0 = 0xFC$; read ceiling $⌈R⌉_1 = 0xF8$ (max priority among
  writers). Both are computed by the RTIC macro and printed at build time
  (`[RTIC] * r @π=252`).
- **Mutex build**: ReaderLow's `lock()` raises the system ceiling (`mintthresh`)
  to $0xFC ≥ π(J)$, so `J` is blocked for the whole read critical section.
- **RW build**: ReaderLow uses `read_lock()`, which only raises the ceiling to
  $0xF8 < π(J)$, so `J` preempts the read critical section.

Periods (µs): ReaderHigh 700, J 1300, ReaderLow 1000, Writer 1500. `J`'s work is
350 µs and its deadline is 400 µs. Under mutex, `WCRT(J) ≈ 1065 µs ≫ 400 µs`;
under rw-lock, `WCRT(J) ≈ 365 µs < 400 µs`. The system is unschedulable under
mutex and schedulable under rw-lock.

## Measurement

Each task reads its own APB timer counter at exec entry/exit. The counter resets
to zero at the compare (= release), so the exit value is the response time
(single-wrap guard). Statistics are recorded per task and printed by the
`Control` teardown along with `J`'s deadline-miss count and a verdict.

## How to run

```sh
# Mutex build (J must be unschedulable)
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic

# RW-lock build (J must be schedulable)
RUNTIME_MS=20 cargo run --release -Frtl-tb,intc-clic,rw
```

Optional compile-time knobs (env vars):

- `RUNTIME_MS` — runtime in ms (required)

Alternatively use `just`:

```sh
just run-mutex
just run-rw
just compare      # runs both and prints a compact side-by-side summary
```

## Expected results (representative, 20 ms runs)

Requires `make verilate INTC=CLIC FULL_UART=1`

|                       | Mutex             | RW-lock       |
| --------------------- | ----------------- | ------------- |
| ReaderHigh worst (µs) | ~715              | ~15           |
| J worst (µs)          | ~1065             | ~365          |
| J deadline misses     | all / ~15         | 0 / ~15       |
| ReaderLow worst (µs)  | ~700              | ~700          |
| Writer worst (µs)     | ~716              | ~16           |
| Verdict               | J NOT schedulable | J schedulable |

`Δ ≈ WCRT(J)_mutex − WCRT(J)_rw ≈ |ReaderLow read CS|`, matching Lemma 2.17 /
Theorem 3.9. Under mutex, J's response time is dominated by ReaderLow's 700 µs
read critical section; under rw-lock, J preempts ReaderLow and its response time
is its own work (350 µs) plus ReaderHigh preemption.

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

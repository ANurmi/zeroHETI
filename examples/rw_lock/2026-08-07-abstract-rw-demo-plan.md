# Readers-writer lock demonstrator for zeroHETI/RTIC

Date: 2026-08-07

## Goal

Empirically demonstrate on the cycle-accurate zeroHETI/Verilator platform that
switching a shared resource from a **mutex** to a **readers-writer lock** lowers
a job's worst-case response time (and can turn an unschedulable system
schedulable) for a task set engineered to satisfy Theorem 3.9's boundary
conditions C1-C5 -- the regime the drone controller (rt_prof) does *not*
exercise.

## Background (from briefing.txt)

- Theorem 3.9 states WCRT of job `J` (Theorem 2.20 / Lemma 2.17) is lowered by
  replacing a mutex on resource `R` with a readers-writer lock **iff** C1-C5:
  - C1: `J` is a reader of `R` and there are >= 2 readers, OR `J` does not access `R`.
  - C2: the highest-preemption-level job accessing `R` is a reader (so read
    ceiling `⌈R⌉1 < ⌈R⌉0`).
  - C3: `π(J) > π(writer_highest)`; if `J ∉ R`, `π(J) <= π(reader_highest)` and a
    lower-priority reader exists.
  - C4: `B(J)` is determined by a lower-priority reader's read critical section of `R`.
  - C5: the second-longest critical section blocking `J` is shorter than the
    longest (or none).
- The RTIC fork (mrtic submodule @ c738be2) already implements readers-writer
  locks: `#[task(..., read = [res])]` -> `read_lock()` raises the system ceiling
  (`mintthresh`) to `read_priority` = max priority among **writers** (`⌈R⌉1`);
  `#[task(..., shared = [res])]` + `.lock()` raises it to `priority` = max
  priority among all users (`⌈R⌉0`, mutex). Both ceilings are computed in
  `rtic-core/src/analysis/mod.rs` (`update_resource_priorities`). No example uses
  `read = [...]` yet.
- The drone controller (examples/rt_prof) runs RTIC on zeroHETI via a prebuilt
  Verilator binary (`build/verilator_build/obj_dir`, built with CLIC),
  `cargo run --release -Frtl-tb,intc-clic` -> `run-sim.sh` -> `make simv`.
  CPU_FREQ_HZ = 10 MHz (rtl-tb). APB timer counter resets to zero at the compare
  event (task release), so response time is readable in-task.

## Task set (satisfies C1-C5)

| Task       | IRQ        | Priority π | Access to R             | Role                                   |
|------------|------------|------------|-------------------------|----------------------------------------|
| ReaderHigh | Timer0Cmp  | `0xFC`     | read (`read_lock`)      | highest-π accessor of R -> **C2**      |
| J          | Timer1Cmp  | `0xFB`     | none                    | measured job -> **C1**, **C3**         |
| ReaderLow  | Timer2Cmp  | `0xF9`     | read, **long** CS       | **C4/C5** (B(J) = this CS)             |
| Writer     | Timer3Cmp  | `0xF8`     | write (`lock`), short CS| **C3**                                 |
| Teardown   | MachineTimer| `0xFF`    | none                    | stop + print summary                   |

- Mutex ceiling `⌈R⌉0 = 0xFC`; read ceiling `⌈R⌉1 = 0xF8`.
- Mutex build: ReaderLow's lock raises the system ceiling to `0xFC >= π(J)=0xFB`,
  so J is blocked by its read critical section.
- RW build: ReaderLow uses `read_lock` -> ceiling `0xF8 < π(J)` -> J preempts it;
  blocking eliminated.
- Prediction: `WCRT(J)_mutex ≈ |RL read CS| + WCET(J) (+ RH preemption)`;
  `WCRT(J)_rw ≈ WCET(J) (+ RH preemption)`. `Δ ≈ |RL read CS|`, engineered to be
  the dominant term.

## Deliverables

1. New crate `examples/rw_lock/` (mirrors rt_prof's build setup): `Cargo.toml`,
   `build.rs`, `.cargo/config.toml` (runner `../run-sim.sh`),
   `riscv32emc-unknown-none-elf.json`, `src/main.rs`, `README.md`.
2. `src/main.rs`: RTIC app with the task set above. **The RTIC fork's parser has
   no `cfg` support on `#[task]` items**, so the `#[cfg_attr(...)]` approach does
   not work (both variants leak through the macro -> duplicate-task E0428).
   Instead: declare each reader **twice** (struct + `impl RticTask`), gated by
   `#[cfg(feature = "rw")]` / `#[cfg(not(feature = "rw"))]`, and extend the mrtic
   parser to evaluate those predicates (see "Hindsight" below).
3. Self-contained WCRT measurement: each periodic task reads its own APB timer
   counter at exec entry/exit (counter resets to 0 at compare = release, so
   `exit_counter` ~= response, with wrap handling `+= period`); tracks per-task
   worst/min response and J's deadline-miss count. MachineTimer teardown prints
   per-task WCRT (us), J's deadline misses, and a PASS/FAIL verdict against a
   compile-time deadline.
4. `README.md` documenting the C1-C5 mapping, ceiling values (confirmed by the
   RTIC macro's `[RTIC] Shared resources @π=` build-time print), how to run both
   configs, and expected results. Plus a small compare script (Justfile or shell).

## Runs

- Mutex: `cargo run --release -Frtl-tb,intc-clic`
- RW:    `cargo run --release -Frtl-tb,intc-clic,rw`

## Implementation steps

0. `source .env`; re-run existing rt_prof to confirm the prebuilt Verilator
   binary + cargo pipeline works.
1. Scaffold the crate (copy config files from rt_prof).
2. Write `main.rs`; build the mutex variant first (debug cycle). **Call
   `ApbUart::init(CPU_FREQ_HZ, 115_200)` before the first `sprintln!`** in
   `#[init]` -- otherwise the mock UART never becomes ready and the firmware
   spins forever polling the TX status register (looks like a hung sim with no
   output).
3. Build the RW variant -- first-ever use of `read=`/`read_lock()`; fix any
   latent RTIC submodule issue (only if necessary) or work around.
4. Tune RL's read-CS length and periods so J's releases reliably overlap RL's CS;
   verify worst case appears in the mutex run. Tuning gotchas:
   - Make the CS **arithmetic-bound**, not memory-bound: zeroHETI's DMEM loads
     are slow (~20 cyc/iter), so a loop that reads `r.data[]` each iteration
     yields a CS of ~1.5-1.8 ms regardless of iteration count -- far longer than
     intended.
   - Keep the CS **comfortably below RL's period** (here ~700 us vs 1000 us). If
     the CS is near/above the period, RL perpetually overruns, `mintthresh` stays
     at the mutex ceiling `0xFC` almost continuously, and the whole system turns
     chaotic: every WCRT inflates and the lowest-priority task (Writer, `0xF8`)
     can starve entirely (0 jobs) -- which is *not* the intended demonstration.
   - Expect Writer to legitimately run few jobs in the mutex build (it is blocked
     by RL's CS for most of the run); that is correct mutex behavior, not a bug.
5. Set J's deadline `D_J` between `WCRT(J)_mutex` and `WCRT(J)_rw`; confirm the
   mutex build misses (unschedulable) while the RW build meets (schedulable).
6. Write README + compare script; capture both outputs.

## Risks

- Read-lock RTIC path unproven: may need a small fix in the mrtic submodule
  (committed there) -- flagged early via an RW-variant build. (In practice the
  read-lock path worked as-is; the submodule fix that *was* needed was cfg
  support in the parser, see "Hindsight".)
- Prebuilt Verilator binary staleness: only matters if the scoreboard were used;
  the firmware-only demo does not depend on it (step 0 verifies the pipeline).
- Worst-case overlap: mitigated by making RL's read CS cover most of its period;
  verified from measured data in step 4. (Careful: *too* long a CS saturates the
  system, see step 4.)
- `CARGO_FEATURE_*` env vars are **not visible to proc macros**: the cfg
  evaluation in the parser must read `RTIC_CFG_FEATURE_<NAME>`, forwarded by
  `build.rs` via `cargo:rustc-env`.

## Verification (after implementation)

- Mutex run: `WCRT(J) ≈ |RL CS| + WCET(J)`, deadline-miss count >= 1.
- RW run: `WCRT(J) ≈ WCET(J)`, 0 misses.
- `WCRT(J)_mutex - WCRT(J)_rw ≈ |RL read CS|`, matching Lemma 2.17 / Theorem 3.9's
  `B` vs `B'`.
- Expect some run-to-run variance (a few jobs / misses); the qualitative result
  (J misses in the mutex build, meets its deadline in the RW build) is stable.

## Hindsight (post-implementation, 2026-08-09)

What actually worked, and what changed:

- **cfg-gating `#[task]` items**: the fork's parser (`rtic-core/src/parser/mod.rs`)
  had no cfg support; both reader variants leaked through the macro as duplicate
  tasks (E0428). Extended the parser to evaluate `#[cfg]` predicates on task
  structs/impls against the env var `RTIC_CFG_FEATURE_<NAME>` (uppercase,
  dashes->underscores), falling back to `CARGO_FEATURE_<NAME>`; `build.rs`
  forwards `CARGO_FEATURE_RW` via `cargo:rustc-env`. Cfg-disabled items are
  emitted as `other_code` with the `task`/`shared`/`idle` pseudo-attrs stripped so
  `rustc` discards them. Both variants build cleanly with identical RTIC
  analysis (`r @π=252`, read ceiling `⌈R⌉1=0xF8` internal).
- **First hang**: firmware spun forever polling the mock UART -- fixed by calling
  `ApbUart::init()` in `#[init]` before any print.
- **Writer "starvation" red herring**: initial `read_busy` was memory-bound
  (~20 cyc/iter DMEM load), producing a ~1.5-1.8 ms CS that exceeded RL's 1000 us
  period. `mintthresh` sat at the 0xFC mutex ceiling almost continuously, so the
  lowest-priority Writer (`0xF8`) never got a window (0 jobs) and every task's
  WCRT inflated. Diagnosis used the CLIC state (irq 23 pending+enabled at level
  248 but never taken) and reading `mintthresh` in teardown. Fixed by an
  arithmetic-only CS (`RL_CS_ITERS=900` -> ~700 us) and confirmed all four tasks
  then run with healthy job counts.
- **Confirmed result** (20 ms runs): mutex `J` worst ~880 us with 6-8/14 deadline
  misses; RW-lock `J` worst ~70 us with 0/15 misses. ReaderHigh drops from ~680
  us to ~29 us (no more blocking on RL's CS). `Δ ≈ |RL CS|` as predicted.
- **Deliverables**: `README.md` + a `Justfile` (`run-mutex`, `run-rw`, `compare`
  which prints a side-by-side paste of both runs).

If redoing the plan: step 4 should explicitly say "keep the CS arithmetic-only
and below the reader's period", and the cfg/task gating approach (parser
extension + `build.rs` env forwarding) should be an explicit deliverable rather
than a discovery made during implementation.

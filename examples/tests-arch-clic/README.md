# CLIC architecture tests for zeroHETI

<ins>Content-note: tests were LLM-generated, then reviewed by heksa.</ins>

Bare-metal RISC-V test binaries that exercise the CLIC (Core Local Interrupt
Controller) of the zeroHETI RT-ibex core (RV32EMC + PULP CLIC v2.0.0, which
implements the obsolete MMIO interrupt interface of an outdated RISC-V CLIC
specification, wired via `rtl/zeroheti_int_ctrl.sv`).
They are adapted from the RISC-V fast-interrupt
[CLIC test plan](https://github.com/riscv/riscv-fast-interrupt/blob/master/test-plan-clic.adoc),
restricted to the M-mode (`smclic`) cases that apply to this core.

## Layout

- `src/lib.rs` — shared harness: CLIC setup, pending,
  threshold and `mnxti` helpers, `Deadline` (MTimer-based watchdog), and
  pass/fail reporting via the simulation backdoor.
- `src/bin/<name>/main.rs` — one binary per test, auto-discovered by Cargo.

## Prerequisites

- The verilated hardware model, built with `make verilate INTC=CLIC FULL_UART=1`
  (run from the repo root; the tests use `sprintln!`, which requires the real
  APB UART).
- The RISC-V toolchain and Verilator available on `PATH` (see repo `README.md`).

## Running the tests

From this directory:

```sh
cargo run --release -Frtl-tb --bin <name>
```

`-Frtl-tb` enables the `bsp/rtl-tb` feature that drives the simulation
pass/fail backdoor (write to `0x380`). The per-target
`runner = "../run-sim.sh"` dispatches to the simulator and formats the ELF.

Run all tests, e.g.:

```sh
for t in nomint-mie nomint-ie nomint-thresh wfi direct; do
  cargo run --release -Frtl-tb --bin "$t"
done
```

A test prints `[<id>] PASSED` and exits 0 on success, `[FAIL]` lines plus a
non-zero exit on failure.

## Test cases and expected statuses

| Binary | Test ID | Purpose | Status |
| --- | --- | --- | --- |
| `nomint-mie` | clicnomint-01 | No interrupt fires while `mstatus.mie = 0` | <font color="green">PASS</font> |
| `nomint-ie` | clicnomint-02 | No interrupt fires while `clicintie = 0` | <font color="green">PASS</font> |
| `nomint-thresh` | clicnomint-03 | No interrupt fires when its level is not above `mintthresh` | <font color="green">PASS</font> |
| `wfi` | clicwfi-01 | `wfi` wakes without trapping when an interrupt is pending and `mie = 0` | <font color="green">PASS</font> |
| `direct` | clicdirect-01 | Direct-mode (`shv = 0`) handler entry, trigger/clear, no re-entry while `mil` held | <font color="yellow">EXPECTED-FAIL</font> |

Every "must-not-fire" test includes a positive-control phase (re-enable the
gate, confirm the interrupt then fires) so it cannot pass spuriously.

### PASS — clicnomint-01 (`nomint-mie`)

Ext0 configured at level 1 with `clicintie = 1`, global `mie = 0`. Pending Ext0
must not be taken: the handler is not called and the pending bit stays set.
Positive control: enabling `mie` lets the still-pending interrupt fire.

### PASS — clicnomint-02 (`nomint-ie`)

With global `mie = 1`, Ext1 (`clicintie = 1`) and Ext0 (`clicintie = 0`) are
pended. Only Ext1 fires; Ext0 stays pending and its handler is not called.
Positive control: enabling Ext0's `clicintie` lets it fire.

### PASS — clicnomint-03 (`nomint-thresh`)

With `mintthresh = 0x40`, Ext0 at level `0x10` and Ext1 at level `0x80` are
pended. Only Ext1 (level above threshold) fires. Positive control: lowering the
threshold to 0 lets the still-pending Ext0 fire.

### PASS — clicwfi-01 (`wfi`)

Ext0 pended with `mie = 0`; `wfi` must wake on the pending interrupt without
trapping (handler not called, pending bit stays set). Positive control:
enabling `mie` lets it fire. Caveat: a hung `wfi` only shows up as a
simulation timeout, not an in-sim failure.

### EXPECTED-FAIL — clicdirect-01 (`direct`)

Direct-mode (`shv = 0`) interrupts are intended to enter via the `mtvec` base
and reach `DefaultHandler`, which checks `mcause` (`irq = 1`, code 27,
`minhv = 0`), `mintstatus.mil`, and that a re-pended Ext0 does not re-enter
while `mil` is held.

**Known rt-ibex limitation:** taking a direct-mode CLIC interrupt glitches the
instruction fetch during the vector redirect — the core fetches a corrupted
copy of the vector-table entry (`0x995fe06c` instead of `0x995fe06f`), which
decodes as an illegal instruction. That exception (`mcause = 0x30000002`,
`irq = 0` / `cause = 2`) preempts the interrupt and lands in `Breakpoint`, so
the test hangs instead of completing.

Reproduced with a bare `loop {}` and with a busy polling loop, and with the
pending bit set before and after `enable_mie()` — i.e. not a timing or test
artifact. The SHV path (`shv = 1`, used by all other tests) is unaffected.
Do not gate CI on this test until the rt-ibex issue is fixed.

## Hardware notes

- Level-only interrupts: 8-bit level in `clicintctl`, no priority bits.
- Nesting rule: an interrupt is taken iff `level > max(mintthresh,
  mintstatus.mil)`; on take `mil = level`, on `mret` it is swapped with
  `mcause.mpil`.
- Edge-triggered `clicintip` auto-clears on core acknowledge.
- `mscratchcsw`/`mscratchcswl` are plain RW storage (no auto-swap), so csw
  signature tests are not meaningful on this core.

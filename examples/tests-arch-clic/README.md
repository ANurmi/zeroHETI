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
for t in nomint-mie nomint-ie nomint-thresh wfi \
         level-preempt level-no-preempt level-thresh-in-handler \
         shv shv-illegal mnxti edge-level; do
  cargo run --release -Frtl-tb --bin "$t"
done
```

A test prints `[<id>] PASSED` and exits 0 on success, `[FAIL]` lines plus a
non-zero exit on failure.

 `direct` is excluded; it is EXPECTED-FAIL, see below.

## Test cases and expected statuses

| Binary | Test ID | Purpose | Status |
| --- | --- | --- | --- |
| `nomint-mie` | clicnomint-01 | No interrupt fires while `mstatus.mie = 0` | <font color="green">PASS</font> |
| `nomint-ie` | clicnomint-02 | No interrupt fires while `clicintie = 0` | <font color="green">PASS</font> |
| `nomint-thresh` | clicnomint-03 | No interrupt fires when its level is not above `mintthresh` | <font color="green">PASS</font> |
| `wfi` | clicwfi-01 | `wfi` wakes without trapping when an interrupt is pending and `mie = 0` | <font color="green">PASS</font> |
| `direct` | clicdirect-01 | Direct-mode (`shv = 0`) handler entry, trigger/clear, no re-entry while `mil` held | <font color="yellow">EXPECTED-FAIL</font> |
| `level-preempt` | cliclevel-02 | Higher-level interrupt preempts a lower-level handler | <font color="gray">???</font> |
| `level-no-preempt` | cliclevel-03 | Lower-level interrupt does not preempt a higher-level handler | <font color="gray">???</font> |
| `level-thresh-in-handler` | cliclevel-04 | An interrupt whose level equals a raised `mintthresh` does not preempt (positive control) | <font color="gray">???</font> |
| `shv` | smclicshv | SHV entry via `mtvt + 4*id`, `mcause.minhv = 1`, edge ip auto-clear | <font color="gray">???</font> |
| `shv-illegal` | smclicshv-illegal | SHV table entry to an illegal instruction raises an illegal-instruction exception, not the handler | <font color="gray">???</font> |
| `mnxti` | smclicmnxti | `mnxti` reports the top pending non-SHV interrupt (peek, `mie = 0`) | <font color="green">PASS</font> |
| `edge-level` | edge-level | Edge re-pend stays pending while `mil` held and re-fires after `mret` | <font color="green">PASS</font> |

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

### ??? — cliclevel-02 (`level-preempt`)

Ext0 (level `0x10`) fires first; its handler asserts Ext1 (level `0x80`).
Since `0x80 > max(mil = 0x10, thresh = 0)`, the CLIC preempts into the Ext1
handler, which records `mcause.code = 28` and `mintstatus.mil = 0x80`, and
`mret` resumes inside the Ext0 handler. The Ext0 handler then redirects `mepc`
to `do_finish`. Checks: both handlers ran exactly once, in visit order
`[Ext0, Ext1]`, Ext1's `mcause`/`mil` are correct.

### ??? — cliclevel-03 (`level-no-preempt`)

Ext0 (level `0x80`) fires first; its handler asserts Ext1 (level `0x10`).
Since `0x10` is not above `max(mil = 0x80, thresh = 0)`, Ext1 must stay
pending and never enter its handler. Checks: Ext0 ran once, Ext1 never ran,
Ext1 remains pending (then unpended before `mret`). Ext1 has a handler
defined only to catch a wrong preemption.

### ??? — cliclevel-04 (`level-thresh-in-handler`)

Ext0 (level `0x10`) fires first; its handler raises `mintthresh` to `0x80`
and asserts Ext1 (level `0x80`). Preemption requires strictly `level >
max(mil, mintthresh)`, so `0x80 == thresh` must not preempt. Positive control:
lowering the threshold to `0x40` lets the same Ext1 preempt. Checks: Ext0 ran
once, Ext1 did not preempt at `0x80 == thresh`, Ext1 preempted exactly once
after the threshold drop.

### ??? — smclicshv (`shv`)

Ext0 configured with `shv = 1`. The CLIC entry jumps to the address stored in
vector-table entry `mtvt + 4*27` (the `_start_Ext0_trap` handler), bypassing
the software trap entry, so `mcause`/`mepc`/`mstatus` reach the handler
untouched. Checks: `mcause.is_interrupt()`, `mcause.code = 27`,
`mcause.minhv = 1`, and the edge `clicintip` pending bit is auto-cleared on
core acknowledge (claim).

### ??? — smclicshv-illegal (`shv-illegal`)

`mtvt` is repointed at a 256-byte-aligned table in DMEM whose Ext0 entry (27)
holds the address of a word containing the illegal encoding `0xFFFFFFFF`.
Taking the SHV interrupt jumps there; executing `0xFFFFFFFF` traps with
`mcause` = illegal instruction (`irq = 0`, `code = 2`), which the exception
dispatcher routes to the `IllegalInstruction` handler. Checks: the exception
handler ran exactly once, `mcause` is not an interrupt with `code = 2`, and
the Ext0 handler never ran. Requires an executable data region: IMEM and DMEM
are both executable on this part, so a plain DMEM static is fetchable.

### PASS — smclicmnxti (`mnxti`)

With `shv = 0` sources at level 1 and `mie = 0` (so nothing is taken),
`mnxti` is a peek that returns `{mtvt[31:8], id << 2}` for the highest-level
pending interrupt and `0` otherwise. Checks: `0` with nothing pending; with
Ext0 (27) pending it encodes id 27 and the `mtvt` base; with Ext0 and Ext1
pending at the same level the tie is resolved to the lowest-indexed source
(Ext0 = 27, matching the `clic_target` tree's tie-break). `rt-ibex` implements
`mnxti` without claim or jump side effects, so the peek never triggers the
direct-mode fetch glitch. The test ends with the two lines latched pending
(an unclaimed edge pending is only cleared by a claim, not by `unpend`, on
this CLIC — so there is no clean way back to "nothing pending").

### PASS — edge-level (`edge-level`)

Ext0 (edge-triggered) fires; the claim clears the edge pending bit. Inside
the handler (`mil = 1`), a fresh `clicintip` edge is pended: it must latch
pending without re-entering. After the handler's `mret` (mil back to `0`), the
still-pending edge is taken again. Checks: no re-entry while `mil = 1`, the
re-pended edge is pending, and it re-fires exactly once after `mret`.

**Coverage hole:** the level-triggered half of this scenario is not testable
via software on this part. `clic_gateway` level mode (`le = 0`) drives `ip`
straight from the external source line and ignores software `clicintip`
writes, and the verilated top ties all external interrupt lines off except I2C
(`verilator/tb/zeroheti_top_wrapper.sv`), so a level cannot be raised or
lowered from the firmware.

## Hardware notes

- Level-only interrupts: 8-bit level in `clicintctl`, no priority bits.
- Nesting rule: an interrupt is taken iff `level > max(mintthresh,
  mintstatus.mil)`; on take `mil = level`, on `mret` it is swapped with
  `mcause.mpil`.
- Edge-triggered (`clicintattr.trig = 1`) `clicintip` is set by a rising edge
  of the source or of the software pending register and auto-clears on core
  acknowledge (claim). In level-sensitive mode the pending bit follows the
  external source and ignores software `clicintip` writes.
- `mnxti` (rt-ibex) is a peek only — no claim or jump side effects — and with
  `CLIC_SHV = 1` it reports only `shv = 0` interrupts.
- `mscratchcsw`/`mscratchcswl` are plain RW storage (no auto-swap), so csw
  signature tests are not meaningful on this core.

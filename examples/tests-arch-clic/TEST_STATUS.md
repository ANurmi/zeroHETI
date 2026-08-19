# Testing status

Work-in-progress tracker for the CLIC architecture test suite. Last updated
2026-08-16.

## Failing tests

The 5 tests below were run but result in failures; README statuses are marked
`???`. Console output was not captured, so the definitive evidence (per-check
`[OK]`/`[FAIL]` lines + `verilator/trace_core_*.log`) needs to be re-collected
at work.

**Task: re-run each failing bin (`cargo run --release -Frtl-tb --bin <name>`),
capture console + trace, and root-cause (test-software bug vs hardware bug)
before changing anything.**

| Binary | Test ID | Notes |
| --- | --- | --- |
| `level-preempt` | cliclevel-02 | uncertain — SW (nested trampoline) or HW (CLIC nesting) |
| `level-no-preempt` | cliclevel-03 | likely SW bug (test) |
| `level-thresh-in-handler` | cliclevel-04 | uncertain — SW (nested trampoline) or HW (CLIC nesting) |
| `shv` | smclicshv | uncertain — SW (`is_pending` read) or HW (`minhv`) |
| `shv-illegal` | smclicshv-illegal | something else — HW-leaning (fetch/decode/exception path) |

### Candidate root causes / observations (UNCONFIRMED, listed per test)

1. **`level-no-preempt`** — likely **test-software bug**:
   `unpend()` clears only the software `clicintip` bit (`sw_i`); an unclaimed
   edge pending in `clic_gateway` is a latch cleared **only by claim**. So after
   handler1's `mret` (mil 0x80 → 0), the still-pended Ext1 (lvl 0x10 > 0) fires,
   and the `"Ext1 never fired"` check fails. Candidate fix: drain/claim the
   pending before `mret`, or restructure the expectation.
2. **`level-preempt` / `level-thresh-in-handler`** — shared nesting machinery
   (`core_interrupt` for Ext0 + `nested_interrupt` for Ext1, spin in handler1,
   `mepc`-redirect to `do_finish`). Check: the `nested_interrupt` trampoline
   does `csrci mstatus, 8` before `mret` (leaves `mie` 0 while handler1 still
   has work — ok here, but verify), and restores `mcause`/`mepc` via CSR
   writes — check interplay with the CLIC `mil`/`mpil` swap on the nested
   `mret`; also whether the preemption happens at all (handler1's `spin_while`
   timeout would fail the test).
3. **`shv`** — candidates: the `"edge ip auto-cleared on take"` check reads
   `is_pending()` after the SHV take; the `clicintip` register is hw-rewritten
   to the gateway `ip` every cycle, but a same-cycle SW-vs-HW write race (arb)
   might leave it set; and/or the `mcause.minhv` (bit 30) check.
4. **`shv-illegal`** — candidates: instruction fetch of the vector-table entry
   and of `0xFFFFFFFF` from **DMEM** (the xbar routes `inst_bus`→DMEM, but the
   SRAM fetch path is unverified); `0xFFFFFFFF` may not decode as illegal in
   this RV32EMC config; exception dispatch to the `IllegalInstruction` symbol
   (the `_dispatch_exception` → `__EXCEPTIONS[2]` path) may not behave as
   assumed.

### Advice for the investigation

- Capture the `[OK]`/`[FAIL]` lines per bin, then
  `verilator/trace_core_*.log` for the failing one.
- Use `DEBUG=1 cargo run --release -Frtl-tb --bin <name>` for waveforms if
  needed.
- Distinguish test-software bugs (e.g. #1) from hardware bugs (rt-ibex / CLIC)
  before touching the tests.
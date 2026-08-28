//! Observability trace backend for zeroHETI tests.
//!
//! RTIC codegen (`rtic-core`) emits calls to four hooks on a user type named
//! in `#[app(..., obs = <path>)]`. This crate is the generic *recording* half
//! of that wiring: a fixed-size, lock-free, timestamped ring buffer. Each test
//! supplies the thin, app-specific *glue*:
//!
//! - the marker type and `impl RticObservability` forwarding to [`obs_push`]
//!   (the generated `TaskId`/`ResourceId` enums are cast to `u8` ids), and
//! - the `task_name` / `res_name` id→name maps passed to [`obs_dump`], so the
//!   dump can print human-readable names without this crate knowing them.
//!
//! Each event occupies two 32-bit words: a compact event word (`kind<<24 |
//! id<<16 | task_prio<<8 | ceiling`) and the low 32 bits of the free-running
//! `mtimer` counter sampled at the hook call. `mtimer` ticks at
//! [`bsp::CPU_FREQ_HZ`], so 2^32 ticks is > 85 s at 50 MHz — runs are
//! milliseconds, making the truncated value unambiguous.
//!
//! The buffer is dumped over UART with [`obs_dump`], which needs the serial
//! port to have been initialized at some point during the app
//! (`ApbUart::init(..)` — see `sprintln`'s requirement in `bsp`).
//!
//! # Concurrency model
//!
//! The hooks are the only writers of the trace, and any of them may run in
//! *any* task context: `on_task_act`/`on_task_comp` wrap every task `exec`
//! (`rtic-core` `wrap_exec_with_obs`), `on_res_acq`/`on_res_rel` wrap each
//! proxy `lock()`/`read_lock()` (before the SRP ceiling is raised / after it
//! is restored, `wrap_lock_fn_with_obs`).
//!
//! On this platform a task handler runs with `mstatus.MIE` set and preemption
//! gated only by the CLIC/ECLIC *level* (`mintthresh`), so a **strictly
//! higher-priority task** may preempt a hook at any instruction boundary, run
//! all of its own (nested) hooks to completion, and only then let the
//! preempted hook resume. Interleavings are therefore:
//!
//! - **single hart** — at most one instruction stream is in flight at a time;
//!   no multi-hart ordering/fence concerns (program order per hart, a coherent
//!   load/store, and aligned 32-bit accesses are atomic on this core);
//! - **strictly LIFO** — a preempting task's whole hook segment runs *between*
//!   two instructions of the preempted hook, then returns;
//! - **bounded nesting** — at most [`OBS_TRACE_CAP`] events total; the tail is
//!   monotonic.
//!
//! # Lock-free append protocol
//!
//! A critical section (`riscv::interrupt::machine::free`) is deliberately *not*
//! used: it would mask interrupts even above the running task's priority,
//! perturbing the timing the hooks exist to observe. Instead the append is a
//! small lock-free protocol that tolerates the one LIFO interleaving that can
//! actually occur. A push is (compiled to plain `lw`/`sw`):
//!
//! ```text
//! idx = OBS_TRACE_LEN                 // read tail
//! if idx < CAP:
//!     OBS_TRACE[idx] = word           // publish
//!     OBS_TRACE_TS[idx] = ts          // publish
//!     OBS_TRACE_LEN = max(LEN, idx+1) // commit, never regress
//! ```
//!
//! If no preemption lands between the tail *read* and the *commit*, every
//! event lands in a unique slot. If a preemption does land in that
//! read-to-commit window, the nested task claims the *same* `idx` and both
//! publish to that one slot (the last writer wins) — but the commit is a `max`,
//! so a stale read can never regress `OBS_TRACE_LEN` or clobber anything beyond
//! the single shared slot. Consequences, which hold unconditionally:
//!
//! - `OBS_TRACE_LEN` is monotonic, ≤ [`OBS_TRACE_CAP`], and every slot `< LEN`
//!   is a fully written, well-formed event; the dump never decodes garbage.
//! - The *only* loss is one event in the rare nested-claim collision above, and
//!   it is bounded (no cascade, no count corruption); on that collision the
//!   slot's word and timestamp may belong to different claimants (microseconds
//!   apart, so the timestamp stays valid either way).
//! - Hooks never disable interrupts and add ~1.4k retired instructions over a
//!   run (~0.7% of active time).
//!
//! # Runtime
//!
//! The teardown dump over UART dominates wall-clock time in the sim (~25 s with
//! `obs` vs ~3 s without at `RUNTIME_MS=10`).

#![no_std]
#![allow(static_mut_refs)]

mod print;

use bsp::mtimer::MTimer;
pub use print::{obs_dump, TsUnit};

/// Maximum number of events the trace can hold. Spends on-target memory (32 bits /
/// 4 bytes per entry).
pub const OBS_TRACE_CAP: usize = 2048;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ObsKind {
    Act = 0,
    Comp = 1,
    Acq = 2,
    Rel = 3,
}

static mut OBS_TRACE_LEN: usize = 0;
static mut OBS_TRACE: [u32; OBS_TRACE_CAP] = [0; OBS_TRACE_CAP];
static mut OBS_TRACE_TS: [u32; OBS_TRACE_CAP] = [0; OBS_TRACE_CAP];

/// Appends one observability event, timestamped by the free-running `mtimer`
/// counter.
///
/// `id` is the app-generated `TaskId`/`ResourceId` discriminator cast to `u8`;
/// `task_prio` is the running task's priority (meaningful for all events);
/// `ceiling` is meaningful only for acquire/release events.
pub fn obs_push(kind: ObsKind, id: u8, task_prio: u16, ceiling: u16) {
    let word = (kind as u32) << 24
        | (id as u32) << 16
        | ((task_prio & 0xff) as u32) << 8
        | (ceiling & 0xff) as u32;
    // Safety: reads of mtimer hi/lo may be disjoint if preempted mid-read, but
    // the result is only a timestamp for tracing. This is a pure read; it does
    // not disturb a `OneShot` used for an app's hyperperiod runtime.
    let ts = MTimer::instance().counter() as u32;
    obs_append(word, ts);
}

/// Lock-free append; see the module docs for the concurrency model.
fn obs_append(word: u32, ts: u32) {
    // Safety: single-hart protocol above; aligned 32-bit accesses are atomic.
    unsafe {
        let idx = OBS_TRACE_LEN;
        if idx < OBS_TRACE_CAP {
            OBS_TRACE[idx] = word;
            OBS_TRACE_TS[idx] = ts;
            OBS_TRACE_LEN = OBS_TRACE_LEN.max(idx + 1);
        }
    }
}

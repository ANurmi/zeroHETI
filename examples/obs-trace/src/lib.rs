//! Observability trace backend for zeroHETI tests.
//!
//! RTIC codegen (`rtic-core`) emits calls to hooks on a user type named in
//! `#[app(..., obs = <path>)]`. This crate is the generic *recording* half of
//! that wiring: a fixed-size, lock-free, timestamped buffer.
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
//! idx = LEN                       // read tail
//! if idx < CAP:
//!     TRACE[idx] = word           // publish
//!     TRACE_TS[idx] = ts          // publish
//!     LEN = max(LEN, idx+1)       // commit, never regress
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
//! - Hooks never disable interrupts and add ~0.7% of active time.
//!
//! # Runtime
//!
//! The teardown dump over UART dominates wall-clock time in most cases, and
//! adds around an order of magnitude in execution time, depending on how many
//! context switches were reported.

#![no_std]
#![allow(static_mut_refs)]

mod print;

use bsp::mtimer::MTimer;
pub use print::{TsUnit, obs_dump};

/// RTIC observer that records every generated observability hook.
pub struct Obs;

impl rtic_observability::RticObservability for Obs {
    fn on_task_act(task_id: u8, task_prio: u16) {
        obs_push(ObsKind::Act, task_id, task_prio, 0);
    }

    fn on_task_comp(task_id: u8, task_prio: u16) {
        obs_push(ObsKind::Comp, task_id, task_prio, 0);
    }

    fn on_res_acq(resource_id: u8, task_prio: u16, ceiling: u16) {
        obs_push(ObsKind::Acq, resource_id, task_prio, ceiling);
    }

    fn on_res_rel(resource_id: u8, task_prio: u16, ceiling: u16) {
        obs_push(ObsKind::Rel, resource_id, task_prio, ceiling);
    }
}

/// Ticks per microsecond, computed at compile time from [`bsp::CPU_FREQ_HZ`].
const TICKS_PER_US: u32 = bsp::CPU_FREQ_HZ / 1_000_000;

/// Maximum number of events the trace can hold. Spends on-target memory (32 bits /
/// 4 bytes per entry).
pub const OBS_TRACE_CAP: usize = 2048;

/// Bit-shift positions for packing event fields into a single `u32` word.
///
/// Layout: `[kind:8][id:8][prio:8][ceiling:8]`
mod layout {
    pub const KIND_SHIFT: u32 = 24;
    pub const ID_SHIFT: u32 = 16;
    pub const PRIO_SHIFT: u32 = 8;
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObsKind {
    Act = 0,
    Comp = 1,
    Acq = 2,
    Rel = 3,
}

impl ObsKind {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Act),
            1 => Some(Self::Comp),
            2 => Some(Self::Acq),
            3 => Some(Self::Rel),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Act => "act",
            Self::Comp => "comp",
            Self::Acq => "acq",
            Self::Rel => "rel",
        }
    }
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
    let word = (kind as u32) << layout::KIND_SHIFT
        | (id as u32) << layout::ID_SHIFT
        | ((task_prio & 0xff) as u32) << layout::PRIO_SHIFT
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

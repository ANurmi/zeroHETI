//! Observability hook backend, wired in via `rtic::app(obs = crate::obs::Obs)`.
//!
//! Each hook appends a compact event word to a fixed-size trace buffer. The
//! buffer is dumped over UART once the simulation runtime has elapsed
//! (`StartSim` teardown).
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
//! - **bounded nesting** — at most `CAP` events total; the tail is monotonic.
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
//! - `OBS_TRACE_LEN` is monotonic, ≤ `CAP`, and every slot `< LEN` is a fully
//!   written, well-formed event; the dump never decodes garbage.
//! - The *only* loss is one event in the rare nested-claim collision above, and
//!   it is bounded (no cascade, no count corruption).
//! - Hooks never disable interrupts and add ~1.4k retired instructions over a
//!   run (~0.7% of active time).
//!
//! # Runtime
//!
//! The teardown dump over UART dominates wall-clock time in the sim (~25 s with
//! the `obs` feature vs ~3 s without at `RUNTIME_MS=10`).
//!
//! # Timestamps
//!
//! Each event records the low 32 bits of the free-running machine timer
//! (`mtimer` counter), sampled right at the hook call. `mtimer` ticks at
//! [`bsp::CPU_FREQ_HZ`], so 2^32 ticks is > 85 s at 50 MHz — runs are
//! milliseconds, making the truncated value unambiguous. The ts is written
//! inline with the event word under the same claim, so under the lock-free
//! protocol a slot's ts/word pair belongs to the same push except in the rare
//! nested-claim collision (where either claimant's pair wins — microseconds
//! apart, the ts stays valid for the slot either way).

use bsp::{mtimer::MTimer, sprintln};

use crate::app::{ResourceId, RticObservability, TaskId};

const OBS_TRACE_CAP: usize = 2048;

#[repr(u8)]
#[derive(Clone, Copy)]
enum ObsKind {
    Act = 0,
    Comp = 1,
    Acq = 2,
    Rel = 3,
}

static mut OBS_TRACE_LEN: u32 = 0;
static mut OBS_TRACE: [u32; OBS_TRACE_CAP] = [0; OBS_TRACE_CAP];
static mut OBS_TRACE_TS: [u32; OBS_TRACE_CAP] = [0; OBS_TRACE_CAP];

pub struct Obs;

impl RticObservability for Obs {
    fn on_task_act(task: TaskId) {
        obs_push(ObsKind::Act, task as u8, 0, 0);
    }
    fn on_task_comp(task: TaskId) {
        obs_push(ObsKind::Comp, task as u8, 0, 0);
    }
    fn on_res_acq(res: ResourceId, task_prio: u16, ceiling: u16) {
        obs_push(ObsKind::Acq, res as u8, task_prio, ceiling);
    }
    fn on_res_rel(res: ResourceId, task_prio: u16, ceiling: u16) {
        obs_push(ObsKind::Rel, res as u8, task_prio, ceiling);
    }
}

fn obs_push(kind: ObsKind, id: u8, task_prio: u16, ceiling: u16) {
    let word = (kind as u32) << 24
        | (id as u32) << 16
        | ((task_prio & 0xff) as u32) << 8
        | (ceiling & 0xff) as u32;
    // Safety: reads of mtimer hi/lo may be disjoint if preempted mid-read, but
    // the result is only a timestamp for tracing. This is a pure read; it does
    // not disturb the OneShot used for the hyperperiod runtime.
    let ts = MTimer::instance().counter() as u32;
    obs_append(word, ts);
}

/// Lock-free append; see the module docs for the concurrency model.
fn obs_append(word: u32, ts: u32) {
    // Safety: single-hart protocol above; aligned 32-bit accesses are atomic.
    unsafe {
        let idx = OBS_TRACE_LEN;
        if (idx as usize) < OBS_TRACE_CAP {
            OBS_TRACE[idx as usize] = word;
            OBS_TRACE_TS[idx as usize] = ts;
            OBS_TRACE_LEN = OBS_TRACE_LEN.max(idx + 1);
        }
    }
}

pub fn obs_dump() {
    // Safety: run at teardown only; all pushes have completed.
    unsafe {
        let n = OBS_TRACE_LEN.min(OBS_TRACE_CAP as u32);
        sprintln!("[obs] trace: {n} events @ mtimer ticks");
        for i in 0..n {
            let w = OBS_TRACE[i as usize];
            let ts = OBS_TRACE_TS[i as usize];
            let kind = (w >> 24) as u8;
            let id = (w >> 16) as u8;
            let prio = (w >> 8) as u8;
            let ceiling = w as u8;
            match kind {
                0 => sprintln!("[obs] @{ts:>10} act  {}", task_name(id)),
                1 => sprintln!("[obs] @{ts:>10} comp {}", task_name(id)),
                2 => sprintln!(
                    "[obs] @{ts:>10} acq  {} t={} c={}",
                    res_name(id),
                    prio,
                    ceiling
                ),
                3 => sprintln!(
                    "[obs] @{ts:>10} rel  {} t={} c={}",
                    res_name(id),
                    prio,
                    ceiling
                ),
                _ => {}
            }
        }
    }
}

fn task_name(id: u8) -> &'static str {
    match id {
        0 => "StartSim",
        1 => "Ctrl0",
        2 => "Ctrl1",
        3 => "Ctrl2",
        4 => "Ctrl3",
        5 => "Mail",
        6 => "Update0",
        7 => "Update1",
        8 => "Update2",
        9 => "Update3",
        10 => "Report0",
        11 => "Report1",
        12 => "Report2",
        13 => "Report3",
        14 => "swd-0xfb",
        15 => "swd-0xf1",
        _ => "?",
    }
}

fn res_name(id: u8) -> &'static str {
    match id {
        0 => "serial",
        1 => "i2c",
        2 => "ibx",
        3 => "obx",
        4 => "mail_buf_0",
        5 => "mail_buf_1",
        6 => "mail_buf_2",
        7 => "mail_buf_3",
        8 => "read_buf_0",
        9 => "read_buf_1",
        10 => "read_buf_2",
        11 => "read_buf_3",
        12 => "ctrl_buf_0",
        13 => "ctrl_buf_1",
        14 => "ctrl_buf_2",
        15 => "ctrl_buf_3",
        _ => "?",
    }
}

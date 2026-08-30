//! Observability trace backend for zeroHETI tests.
//!
//! RTIC codegen (`rtic-core`) emits calls to hooks on a user type named in
//! `#[app(..., obs = <path>)]`. This crate is the generic *recording* half of
//! that wiring: a fixed-size, lock-free, timestamped buffer.
//!
//! Call to print the trace:
//!
//! ```
//! obs_trace::obs_dump!(obs_trace::TsUnit::Micros);
//! ```
//!
//! # Runtime
//!
//! The teardown dump over UART dominates wall-clock time in most cases, and
//! adds around an order of magnitude in execution time, depending on how many
//! context switches were reported.

mod print;

pub use print::{ObsPrinter, TsUnit};

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
const TICKS_PER_US: u32 = crate::CPU_FREQ_HZ / 1_000_000;

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

/// Appends one observability event, timestamped by `mtimer`
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
    let ts = crate::mtimer::MTimer::instance().counter() as u32;
    obs_append(word, ts);
}

/// Lock-free append; see the module docs for the concurrency model.
fn obs_append(word: u32, ts: u32) {
    // Protect access to shared data
    use crate::register::mintthresh::{self, Mintthresh};
    let old_thr = mintthresh::write(Mintthresh::from(0xff));

    // Safety: shared access protected by above mintthresh
    unsafe {
        let idx = OBS_TRACE_LEN;
        if idx < OBS_TRACE_CAP {
            OBS_TRACE[idx] = word;
            OBS_TRACE_TS[idx] = ts;
            OBS_TRACE_LEN = OBS_TRACE_LEN.max(idx + 1);
        }
    }

    mintthresh::write(Mintthresh::from(old_thr));
}

use bsp::sprintln;

use crate::{OBS_TRACE, OBS_TRACE_CAP, OBS_TRACE_LEN, OBS_TRACE_TS};

/// Ticks per microsecond, computed at compile time from [`bsp::CPU_FREQ_HZ`].
const TICKS_PER_US: u32 = bsp::CPU_FREQ_HZ / 1_000_000;

/// Selects the timestamp unit printed by [`obs_dump`].
#[derive(Clone, Copy)]
pub enum TsUnit {
    /// Raw mtimer ticks (cycles).
    Cycles,
    /// Microseconds (ticks divided by `CPU_FREQ_HZ / 1_000_000`).
    Micros,
}

/// Dumps the trace over UART.
///
/// `task_name` and `res_name` map an event id to a human-readable name and are
/// supplied by the specific test (this crate has no knowledge of any app's
/// task/resource layout).
///
/// `ts_unit` selects whether timestamps are printed as raw cycles or
/// microseconds.
pub fn obs_dump(
    task_name: fn(u8) -> &'static str,
    res_name: fn(u8) -> &'static str,
    ts_unit: TsUnit,
) {
    // Safety: run at teardown only; all pushes have completed.
    unsafe {
        let n_events = OBS_TRACE_LEN.min(OBS_TRACE_CAP);
        header(n_events, ts_unit);
        sprintln!("[obs] [TRACE_START]");
        for i in 0..n_events {
            let w = OBS_TRACE[i];
            let ts = match ts_unit {
                TsUnit::Cycles => OBS_TRACE_TS[i],
                TsUnit::Micros => OBS_TRACE_TS[i] / TICKS_PER_US,
            };
            let kind = (w >> 24) as u8;
            let id = (w >> 16) as u8;
            let prio = (w >> 8) as u8;
            let ceiling = w as u8;
            match kind {
                0 => sprintln!("[obs] @{ts:>10} act  {} t={}", task_name(id), prio),
                1 => sprintln!("[obs] @{ts:>10} comp {} t={}", task_name(id), prio),
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
        sprintln!("[obs] [TRACE_END]");
    }
}

fn header(n_events: usize, ts_unit: TsUnit) {
    let (label, ts) = match ts_unit {
        TsUnit::Cycles => ("mtimer ticks", "ts (ticks)"),
        TsUnit::Micros => ("us", "ts (us)"),
    };
    sprintln!("[obs] trace: {n_events} events @ {label}");
    let event_kind = "type";
    let rest = "name/params";
    sprintln!("[obs] @{ts:>8} act  {event_kind} {rest}");
}

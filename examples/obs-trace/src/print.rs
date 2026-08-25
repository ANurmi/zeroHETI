use bsp::sprintln;

use crate::{OBS_TRACE, OBS_TRACE_CAP, OBS_TRACE_LEN, OBS_TRACE_TS};

/// Dumps the trace over UART.
///
/// `task_name` and `res_name` map an event id to a human-readable name and are
/// supplied by the specific test (this crate has no knowledge of any app's
/// task/resource layout).
pub fn obs_dump(task_name: fn(u8) -> &'static str, res_name: fn(u8) -> &'static str) {
    // Safety: run at teardown only; all pushes have completed.
    unsafe {
        let n_events = OBS_TRACE_LEN.min(OBS_TRACE_CAP);
        header(n_events);
        sprintln!("[obs] [TRACE_START]");
        for i in 0..n_events {
            let w = OBS_TRACE[i];
            let ts = OBS_TRACE_TS[i];
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
        sprintln!("[obs] [TRACE_END]");
    }
}

fn header(n_events: usize) {
    // Overview
    sprintln!("[obs] trace: {n_events} events @ mtimer ticks");

    let ts = "ts (ticks)";
    let event_kind = "type";
    let rest = "name/params";
    // Column names
    sprintln!("[obs] @{ts:>8} act  {event_kind} {rest}");
}

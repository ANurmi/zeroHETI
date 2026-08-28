use bsp::{mtimer::MTimer, sprintln};

use crate::{OBS_TRACE, OBS_TRACE_CAP, OBS_TRACE_LEN, OBS_TRACE_TS, ObsKind, TICKS_PER_US, layout};

/// Selects the timestamp unit printed by [`obs_dump`].
#[derive(Clone, Copy)]
pub enum TsUnit {
    /// CPU clock cycles (mtimer ticks multiplied by the clock divider).
    Cycles,
    /// Microseconds (mtimer ticks scaled to CPU cycles, then to microseconds).
    Micros,
}

pub struct ObsPrinter {
    task_name: fn(u8) -> &'static str,
    res_name: fn(u8) -> &'static str,
    ts_unit: TsUnit,
    clkdiv: u32,
    name_width: usize,
}

impl ObsPrinter {
    pub fn new(
        task_name: fn(u8) -> &'static str,
        res_name: fn(u8) -> &'static str,
        ts_unit: TsUnit,
    ) -> Self {
        let n_events = unsafe { OBS_TRACE_LEN.min(OBS_TRACE_CAP) };
        let mut name_width = 4;
        for i in 0..n_events {
            let word = unsafe { OBS_TRACE[i] };
            let kind_raw = (word >> layout::KIND_SHIFT) as u8;
            let id = (word >> layout::ID_SHIFT) as u8;
            if let Some(kind) = ObsKind::from_u8(kind_raw) {
                let name = match kind {
                    ObsKind::Act | ObsKind::Comp => task_name(id),
                    ObsKind::Acq | ObsKind::Rel => res_name(id),
                };
                name_width = name_width.max(name.len());
            }
        }

        Self {
            task_name,
            res_name,
            ts_unit,
            clkdiv: MTimer::instance().clkdiv(),
            name_width,
        }
    }

    fn timestamp(&self, ticks: u32) -> u32 {
        match self.ts_unit {
            TsUnit::Cycles => ticks * self.clkdiv,
            TsUnit::Micros => ticks * self.clkdiv / TICKS_PER_US,
        }
    }

    fn print_event(&self, word: u32, ticks: u32) {
        let ts = self.timestamp(ticks);
        let kind_raw = (word >> layout::KIND_SHIFT) as u8;
        let id = (word >> layout::ID_SHIFT) as u8;
        let prio = (word >> layout::PRIO_SHIFT) as u8;
        let ceiling = word as u8;
        let Some(kind) = ObsKind::from_u8(kind_raw) else {
            return;
        };

        match kind {
            ObsKind::Act | ObsKind::Comp => sprintln!(
                "[obs] @{0:>10} {1:<4} {2:<3$} t={4}",
                ts,
                kind.as_str(),
                (self.task_name)(id),
                self.name_width,
                prio
            ),
            ObsKind::Acq | ObsKind::Rel => sprintln!(
                "[obs] @{0:>10} {1:<4} {2:<3$} t={4} c={5}",
                ts,
                kind.as_str(),
                (self.res_name)(id),
                self.name_width,
                prio,
                ceiling
            ),
        }
    }

    fn header(&self, n_events: usize) {
        let (label, ts_label) = match self.ts_unit {
            TsUnit::Cycles => ("cc", "ts (cc)"),
            TsUnit::Micros => ("us", "ts (us)"),
        };
        sprintln!("[obs] trace: {n_events} events @ {label}");
        sprintln!("[obs] @{ts_label:>10} type  name/params");
    }

    pub fn dump(&self) {
        let n_events = unsafe { OBS_TRACE_LEN.min(OBS_TRACE_CAP) };
        sprintln!("[obs] [TRACE_START]");
        self.header(n_events);
        if n_events == OBS_TRACE_CAP {
            sprintln!("[obs] warning: trace capacity reached; subsequent events were dropped");
        }
        for i in 0..n_events {
            self.print_event(unsafe { OBS_TRACE[i] }, unsafe { OBS_TRACE_TS[i] });
        }
        sprintln!("[obs] [TRACE_END]");
    }
}

/// Dumps the trace over UART.
///
/// `task_name` and `res_name` map an event id to a human-readable name and are
/// supplied by the specific test (this crate has no knowledge of any app's
/// task/resource layout).
///
/// `ts_unit` selects whether timestamps are printed as CPU clock cycles or
/// microseconds.
#[macro_export]
macro_rules! obs_dump {
    ($unit:expr) => {
        // Leverages RTIC generated symbols `task_name` and `res_name`
        obs_trace::ObsPrinter::new(
            crate::app::task_name,
            crate::app::res_name,
            obs_trace::TsUnit::Micros,
        )
        .dump()
    };
}

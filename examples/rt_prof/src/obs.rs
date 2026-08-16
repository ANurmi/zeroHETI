//! Observability hook backend, wired in via `rtic::app(obs = crate::obs::Obs)`.
//!
//! Each hook appends a compact event word to a fixed-size trace buffer
//! (cheap: one atomic load + store under a critical section, no I/O), so
//! the hooks stay timing-neutral for the benchmark. The buffer is dumped
//! over UART once the simulation runtime has elapsed (`StartSim` teardown).

use bsp::riscv;
use bsp::sprintln;

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

use core::sync::atomic::{AtomicU32, Ordering};
static OBS_TRACE_LEN: AtomicU32 = AtomicU32::new(0);
static mut OBS_TRACE: [u32; OBS_TRACE_CAP] = [0; OBS_TRACE_CAP];

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
    riscv::interrupt::machine::free(|| obs_push_unlocked(word));
}

fn obs_push_unlocked(word: u32) {
    let idx = OBS_TRACE_LEN.load(Ordering::Relaxed) as usize;
    if idx < OBS_TRACE_CAP {
        // Safety: each idx is written at most once, `static_mut_refs` allowed
        unsafe { OBS_TRACE[idx] = word };
        OBS_TRACE_LEN.store((idx + 1) as u32, Ordering::Relaxed);
    }
}

pub fn obs_dump() {
    let n = OBS_TRACE_LEN
        .load(Ordering::Relaxed)
        .min(OBS_TRACE_CAP as u32);
    sprintln!("[obs] trace: {n} events");
    for i in 0..n {
        // Safety: read slots that were written by `obs_push`
        let w = unsafe { OBS_TRACE[i as usize] };
        let kind = (w >> 24) as u8;
        let id = (w >> 16) as u8;
        let prio = (w >> 8) as u8;
        let ceiling = w as u8;
        match kind {
            0 => sprintln!("[obs] act   {}", task_name(id)),
            1 => sprintln!("[obs] comp  {}", task_name(id)),
            2 => sprintln!("[obs] acq   {} t={} c={}", res_name(id), prio, ceiling),
            3 => sprintln!("[obs] rel   {} t={} c={}", res_name(id), prio, ceiling),
            _ => {}
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

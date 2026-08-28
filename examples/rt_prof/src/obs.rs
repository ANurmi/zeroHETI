//! rt_prof-specific observability glue.
//!
//! The generic, timestamped trace backend lives in the reusable `obs_trace`
//! crate. This module supplies the app-specific parts it needs:
//!
//! - the marker type `Obs` and its [`RticObservability`] impl, forwarding the
//!   generated `TaskId`/`ResourceId` enums (cast to `u8` ids) into
//!   [`obs_trace::obs_push`], and
//! - the [`task_name`] / [`res_name`] id -> name maps for
//!   [`obs_trace::obs_dump`].

use crate::app::{ResourceId, RticObservability, TaskId};
use obs_trace::ObsKind;

pub struct Obs;

impl RticObservability for Obs {
    fn on_task_act(task: TaskId, task_prio: u16) {
        obs_trace::obs_push(ObsKind::Act, task as u8, task_prio, 0);
    }
    fn on_task_comp(task: TaskId, task_prio: u16) {
        obs_trace::obs_push(ObsKind::Comp, task as u8, task_prio, 0);
    }
    fn on_res_acq(res: ResourceId, task_prio: u16, ceiling: u16) {
        obs_trace::obs_push(ObsKind::Acq, res as u8, task_prio, ceiling);
    }
    fn on_res_rel(res: ResourceId, task_prio: u16, ceiling: u16) {
        obs_trace::obs_push(ObsKind::Rel, res as u8, task_prio, ceiling);
    }
}

pub fn obs_dump() {
    obs_trace::obs_dump(task_name, res_name);
}

fn task_name(id: u8) -> &'static str {
    // N.b., make sure to keep these updated
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

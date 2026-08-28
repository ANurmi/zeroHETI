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
    obs_trace::obs_dump(crate::app::task_name, crate::app::res_name, obs_trace::TsUnit::Micros);
}

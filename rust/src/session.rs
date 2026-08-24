//! Stateful plant session for APC (HMPC) closed-loop stepping at control period `Ts`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::closed_loop::{ControllerMask, PlantWideController};
use crate::process::{default_delta_t, TennesseeEastmanProcess, N_IDV};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionConfig {
    #[serde(default = "default_seed")]
    pub seed: f64,
    /// Integrator steps per APC call (typically `Ts` in seconds).
    #[serde(default = "default_integrate_steps")]
    pub integrate_steps: usize,
    #[serde(default)]
    pub setpoints: BTreeMap<usize, f64>,
    #[serde(default)]
    pub held_setpoints: Vec<usize>,
    #[serde(default)]
    pub loop_mode: BTreeMap<usize, crate::experiment::LoopMode>,
}

fn default_seed() -> f64 {
    crate::process::DEFAULT_RNG_SEED
}

fn default_integrate_steps() -> usize {
    60
}

#[derive(Clone, Debug, Serialize)]
pub struct StepResponse {
    pub step: usize,
    pub time_s: u32,
    pub xmeas: Vec<f64>,
    pub xmv: Vec<f64>,
    pub setpt: Vec<f64>,
    pub shutdown: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shutdown_reasons: Option<Vec<String>>,
}

pub struct PlantSession {
    process: TennesseeEastmanProcess,
    ctrl: PlantWideController,
    step: usize,
    held: Vec<(usize, f64)>,
    mask: ControllerMask,
    integrate_steps: usize,
}

impl PlantSession {
    pub fn new(cfg: &SessionConfig) -> Self {
        let dt = default_delta_t();
        let mut process = TennesseeEastmanProcess::with_seed(cfg.seed);
        process.teinit();
        for i in 0..N_IDV {
            process.set_idv(i + 1, false);
        }
        let mut ctrl = PlantWideController::new(dt);
        PlantWideController::apply_base_xmv(&mut process);
        let overrides: Vec<(usize, f64)> = cfg.setpoints.iter().map(|(&n, &v)| (n, v)).collect();
        ctrl.apply_setpoint_overrides(&overrides);
        let held: Vec<(usize, f64)> = cfg
            .held_setpoints
            .iter()
            .map(|&n| {
                let value = cfg.setpoints.get(&n).copied().unwrap_or(ctrl.setpt[n - 1]);
                (n, value)
            })
            .collect();
        let manual: BTreeSet<usize> = cfg
            .loop_mode
            .iter()
            .filter(|(_, m)| **m == crate::experiment::LoopMode::Manual)
            .map(|(&n, _)| n)
            .collect();
        Self {
            process,
            ctrl,
            step: 0,
            held,
            mask: ControllerMask::from_manual_setpoints(&manual),
            integrate_steps: cfg.integrate_steps.max(1),
        }
    }

    /// Advance the plant by `integrate_steps` (or config default) and apply APC setpoint writes.
    pub fn step_apc(&mut self, setpoint_writes: &BTreeMap<usize, f64>) -> StepResponse {
        for (&n, &v) in setpoint_writes {
            if (1..=20).contains(&n) {
                self.ctrl.setpt[n - 1] = v;
            }
        }
        for _ in 0..self.integrate_steps {
            self.step += 1;
            self.ctrl.step_masked(&mut self.process, self.step, &self.mask);
            for &(n, value) in &self.held {
                if !setpoint_writes.contains_key(&n) {
                    self.ctrl.setpt[n - 1] = value;
                }
            }
            self.process.integrate(default_delta_t());
            self.ctrl.constrain_hand(&mut self.process);
            if self.process.is_shutdown() {
                break;
            }
        }
        self.snapshot()
    }

    pub fn snapshot(&self) -> StepResponse {
        StepResponse {
            step: self.step,
            time_s: self.step as u32,
            xmeas: self.process.xmeas().iter().copied().collect(),
            xmv: self.process.xmv().iter().copied().collect(),
            setpt: self.ctrl.setpt.to_vec(),
            shutdown: self.process.is_shutdown(),
            shutdown_reasons: if self.process.is_shutdown() {
                Some(
                    self.process
                        .shutdown_reasons()
                        .into_iter()
                        .map(str::to_string)
                        .collect(),
                )
            } else {
                None
            },
        }
    }

    pub fn set_idv(&mut self, idv: usize, on: bool) {
        if (1..=N_IDV).contains(&idv) {
            self.process.set_idv(idv, on);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{N_XMEAS, N_XMV};

    #[test]
    fn session_steps_without_shutdown() {
        let mut s = PlantSession::new(&SessionConfig {
            seed: default_seed(),
            integrate_steps: 120,
            setpoints: BTreeMap::new(),
            held_setpoints: vec![18],
            loop_mode: BTreeMap::new(),
        });
        let mut writes = BTreeMap::new();
        writes.insert(18, 121.0);
        let r = s.step_apc(&writes);
        assert_eq!(r.step, 120);
        assert!(!r.shutdown);
        assert_eq!(r.xmeas.len(), N_XMEAS);
        assert_eq!(r.xmv.len(), N_XMV);
    }
}

//! Run open- or closed-loop simulations for the local console.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::closed_loop::PlantWideController;
use crate::open_loop::StripperLevelController;
use crate::process::{
    default_delta_t, TennesseeEastmanProcess, DEFAULT_RNG_SEED, N_IDV, N_XMEAS, N_XMV,
};

pub const MAX_NPTS: usize = 345_600;
pub const MAX_SAMPLES: usize = 4_096;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SimMode {
    #[default]
    ClosedLoop,
    OpenLoop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Injection {
    /// 1-based `IDV(n)`.
    pub idv: usize,
    /// 1-based integrator step at which the disturbance turns on (`I` in Fortran).
    pub start_step: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationRequest {
    #[serde(default)]
    pub mode: SimMode,
    pub npts: usize,
    #[serde(default = "default_record_every")]
    pub record_every: usize,
    #[serde(default = "default_seed")]
    pub seed: f64,
    /// 1-based `SETPT(n)` → value. Closed-loop only.
    #[serde(default)]
    pub setpoints: BTreeMap<usize, f64>,
    /// 1-based `SETPT(n)` restored after each controller step so cascade cannot overwrite it.
    #[serde(default)]
    pub held_setpoints: Vec<usize>,
    #[serde(default)]
    pub injections: Vec<Injection>,
    /// 1-based `XMV(n)` overrides after `TEINIT`. Open-loop only.
    #[serde(default)]
    pub open_loop_xmv: BTreeMap<usize, f64>,
    /// Stripper-level setpoint for the single PI in `temain.f`. Open-loop only.
    #[serde(default)]
    pub open_loop_stripper_sp: Option<f64>,
}

fn default_record_every() -> usize {
    60
}

fn default_seed() -> f64 {
    DEFAULT_RNG_SEED
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulationResult {
    pub mode: SimMode,
    pub npts: usize,
    pub steps_run: usize,
    pub record_every: usize,
    pub delta_t_hours: f64,
    pub seed: f64,
    pub time_s: Vec<u32>,
    pub xmeas: Vec<Vec<f32>>,
    pub xmv: Vec<Vec<f32>>,
    /// Present when recorded (closed-loop experiments / console export).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setpt: Option<Vec<Vec<f32>>>,
    pub shutdown: bool,
    pub shutdown_time_s: Option<u32>,
    pub shutdown_reasons: Vec<String>,
    pub injections: Vec<Injection>,
}

#[derive(Clone, Debug)]
pub struct SimulationError(pub String);

impl SimulationRequest {
    pub fn validate(&self) -> Result<usize, SimulationError> {
        if self.npts == 0 || self.npts > MAX_NPTS {
            return Err(SimulationError(format!(
                "npts must be 1..={MAX_NPTS}, got {}",
                self.npts
            )));
        }
        for inj in &self.injections {
            if !(1..=N_IDV).contains(&inj.idv) {
                return Err(SimulationError(format!(
                    "IDV index must be 1..={N_IDV}, got {}",
                    inj.idv
                )));
            }
        }
        for &n in self.setpoints.keys().chain(self.held_setpoints.iter()) {
            if !(1..=20).contains(&n) {
                return Err(SimulationError(format!(
                    "SETPT index must be 1..=20, got {n}"
                )));
            }
        }
        for &n in self.open_loop_xmv.keys() {
            if !(1..=N_XMV).contains(&n) {
                return Err(SimulationError(format!(
                    "XMV index must be 1..={N_XMV}, got {n}"
                )));
            }
        }
        Ok(effective_record_every(self.npts, self.record_every.max(1)))
    }
}

pub fn effective_record_every(npts: usize, record_every: usize) -> usize {
    let every = record_every.max(1);
    let max_intervals = MAX_SAMPLES.saturating_sub(1).max(1);
    let min_every = npts.div_ceil(max_intervals).max(1);
    every.max(min_every)
}

pub fn run(req: &SimulationRequest) -> Result<SimulationResult, SimulationError> {
    let record_every = req.validate()?;
    let dt = default_delta_t();
    let mut process = TennesseeEastmanProcess::with_seed(req.seed);
    process.teinit();
    for i in 0..N_IDV {
        process.set_idv(i + 1, false);
    }

    match req.mode {
        SimMode::ClosedLoop => run_closed(req, &mut process, dt, record_every),
        SimMode::OpenLoop => run_open(req, &mut process, dt, record_every),
    }
}

fn run_closed(
    req: &SimulationRequest,
    process: &mut TennesseeEastmanProcess,
    dt: f64,
    record_every: usize,
) -> Result<SimulationResult, SimulationError> {
    let mut ctrl = PlantWideController::new(dt);
    PlantWideController::apply_base_xmv(process);
    let overrides: Vec<(usize, f64)> = req.setpoints.iter().map(|(&n, &v)| (n, v)).collect();
    ctrl.apply_setpoint_overrides(&overrides);
    let held: Vec<(usize, f64)> = req
        .held_setpoints
        .iter()
        .map(|&n| {
            let value = req.setpoints.get(&n).copied().unwrap_or(ctrl.setpt[n - 1]);
            (n, value)
        })
        .collect();

    let record_setpt = true;
    let mut rec = Recorder::new(req, record_every, record_setpt);
    rec.push(0, process, Some(&ctrl));

    for i in 1..=req.npts {
        apply_injections(process, &req.injections, i);
        ctrl.step(process, i);
        for &(n, value) in &held {
            ctrl.setpt[n - 1] = value;
        }
        if should_record_step(i, record_every, &req.injections) {
            rec.push_if_new(i as u32, process, Some(&ctrl));
        }
        process.integrate(dt);
        ctrl.constrain_hand(process);
        if process.is_shutdown() {
            rec.note_shutdown(i as u32, process);
            rec.push_if_new(i as u32, process, Some(&ctrl));
            break;
        }
    }

    Ok(rec.finish(req, record_every))
}

fn run_open(
    req: &SimulationRequest,
    process: &mut TennesseeEastmanProcess,
    dt: f64,
    record_every: usize,
) -> Result<SimulationResult, SimulationError> {
    process.set_xmv(10, 38.0);
    for (&n, &value) in &req.open_loop_xmv {
        process.set_xmv(n, value);
    }
    let mut ctrl = StripperLevelController::from_process(process);
    if let Some(sp) = req.open_loop_stripper_sp {
        ctrl.setpt = sp;
    }

    let mut rec = Recorder::new(req, record_every, false);
    rec.push(0, process, None);

    for i in 1..=req.npts {
        apply_injections(process, &req.injections, i);
        ctrl.apply(process, dt);
        if should_record_step(i, record_every, &req.injections) {
            rec.push_if_new(i as u32, process, None);
        }
        process.integrate(dt);
        if process.is_shutdown() {
            rec.note_shutdown(i as u32, process);
            rec.push_if_new(i as u32, process, None);
            break;
        }
    }

    Ok(rec.finish(req, record_every))
}

pub(crate) fn apply_injections(
    process: &mut TennesseeEastmanProcess,
    injections: &[Injection],
    step: usize,
) {
    for inj in injections {
        if step >= inj.start_step.max(1) {
            process.set_idv(inj.idv, true);
        }
    }
}

pub(crate) fn should_record_step(step: usize, record_every: usize, injections: &[Injection]) -> bool {
    if step % record_every == 0 {
        return true;
    }
    injections.iter().any(|inj| inj.start_step == step)
}

struct Recorder {
    mode: SimMode,
    npts: usize,
    seed: f64,
    time_s: Vec<u32>,
    xmeas: Vec<Vec<f32>>,
    xmv: Vec<Vec<f32>>,
    setpt: Option<Vec<Vec<f32>>>,
    shutdown: bool,
    shutdown_time_s: Option<u32>,
    shutdown_reasons: Vec<String>,
    injections: Vec<Injection>,
    steps_run: usize,
}

impl Recorder {
    fn new(req: &SimulationRequest, record_every: usize, record_setpt: bool) -> Self {
        let cap = req.npts / record_every + 2;
        Self {
            mode: req.mode,
            npts: req.npts,
            seed: req.seed,
            time_s: Vec::with_capacity(cap),
            xmeas: vec![Vec::with_capacity(cap); N_XMEAS],
            xmv: vec![Vec::with_capacity(cap); N_XMV],
            setpt: if record_setpt {
                Some(vec![Vec::with_capacity(cap); 20])
            } else {
                None
            },
            shutdown: false,
            shutdown_time_s: None,
            shutdown_reasons: Vec::new(),
            injections: req.injections.clone(),
            steps_run: 0,
        }
    }

    fn push(
        &mut self,
        t: u32,
        process: &TennesseeEastmanProcess,
        ctrl: Option<&PlantWideController>,
    ) {
        self.time_s.push(t);
        self.steps_run = t as usize;
        let x = process.xmeas();
        let mv = process.xmv();
        for i in 0..N_XMEAS {
            self.xmeas[i].push(x[i] as f32);
        }
        for i in 0..N_XMV {
            self.xmv[i].push(mv[i] as f32);
        }
        if let (Some(setpt), Some(ctrl)) = (&mut self.setpt, ctrl) {
            for i in 0..20 {
                setpt[i].push(ctrl.setpt[i] as f32);
            }
        }
    }

    fn push_if_new(
        &mut self,
        t: u32,
        process: &TennesseeEastmanProcess,
        ctrl: Option<&PlantWideController>,
    ) {
        if self.time_s.last().copied() == Some(t) {
            return;
        }
        self.push(t, process, ctrl);
    }

    fn note_shutdown(&mut self, t: u32, process: &TennesseeEastmanProcess) {
        self.shutdown = true;
        self.shutdown_time_s = Some(t);
        self.shutdown_reasons = process
            .shutdown_reasons()
            .into_iter()
            .map(str::to_string)
            .collect();
        self.steps_run = t as usize;
    }

    fn finish(mut self, req: &SimulationRequest, record_every: usize) -> SimulationResult {
        if !self.shutdown {
            self.steps_run = req.npts;
        }
        SimulationResult {
            mode: self.mode,
            npts: self.npts,
            steps_run: self.steps_run,
            record_every,
            delta_t_hours: default_delta_t(),
            seed: self.seed,
            time_s: self.time_s,
            xmeas: self.xmeas,
            xmv: self.xmv,
            setpt: self.setpt,
            shutdown: self.shutdown,
            shutdown_time_s: self.shutdown_time_s,
            shutdown_reasons: self.shutdown_reasons,
            injections: self.injections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injection_start_step_is_always_recorded() {
        let req = SimulationRequest {
            mode: SimMode::ClosedLoop,
            npts: 600,
            record_every: 100,
            seed: DEFAULT_RNG_SEED,
            setpoints: BTreeMap::new(),
            held_setpoints: Vec::new(),
            injections: vec![Injection {
                idv: 12,
                start_step: 300,
            }],
            open_loop_xmv: BTreeMap::new(),
            open_loop_stripper_sp: None,
        };
        let out = run(&req).expect("sim");
        assert!(
            out.time_s.contains(&300),
            "injection at step 300 must be recorded: {:?}",
            out.time_s
        );
    }

    #[test]
    fn closed_loop_records_t0_and_grid() {
        let req = SimulationRequest {
            mode: SimMode::ClosedLoop,
            npts: 600,
            record_every: 100,
            seed: DEFAULT_RNG_SEED,
            setpoints: BTreeMap::new(),
            held_setpoints: Vec::new(),
            injections: vec![Injection {
                idv: 12,
                start_step: 300,
            }],
            open_loop_xmv: BTreeMap::new(),
            open_loop_stripper_sp: None,
        };
        let out = run(&req).expect("sim");
        assert_eq!(out.time_s.first().copied(), Some(0));
        assert_eq!(out.time_s.last().copied(), Some(600));
        assert_eq!(out.xmeas.len(), N_XMEAS);
        assert_eq!(out.xmeas[8].len(), out.time_s.len());
        assert!(!out.shutdown);
    }

    #[test]
    fn record_every_caps_samples() {
        assert_eq!(effective_record_every(100, 10), 10);
        let every = effective_record_every(MAX_NPTS, 1);
        assert!(MAX_NPTS / every + 1 <= MAX_SAMPLES + 1);
    }
}

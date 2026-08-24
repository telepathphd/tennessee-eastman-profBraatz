//! Identification experiments: schedules, loop Auto/Manual, mimo-sim CSV export.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::closed_loop::{ControllerMask, PlantWideController};
use crate::open_loop::StripperLevelController;
use crate::process::{
    default_delta_t, TennesseeEastmanProcess, N_IDV, N_XMEAS, N_XMV,
};
use crate::simulate::{
    apply_injections, effective_record_every, should_record_step, Injection, SimMode,
    SimulationError, SimulationRequest, MAX_NPTS,
};

pub const CSV_TIME_BASE: &str = "2026-03-26 00:00:00";

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MvChannelKind {
    Setpoint,
    Xmv,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SchedulePoint {
    /// 1-based integrator step at which this value applies (`>= start_step`).
    pub start_step: usize,
    /// 1-based `SETPT(n)` or `XMV(n)` depending on schedule type.
    pub n: usize,
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MvChannel {
    pub kind: MvChannelKind,
    /// 1-based `SETPT(n)` or `XMV(n)`.
    pub n: usize,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CvChannel {
    /// 1-based `XMEAS(n)`.
    pub n: usize,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExperimentRequest {
    #[serde(flatten)]
    pub sim: SimulationRequest,
    #[serde(default)]
    pub setpoint_schedule: Vec<SchedulePoint>,
    #[serde(default)]
    pub xmv_schedule: Vec<SchedulePoint>,
    /// 1-based `SETPT(n)` → Auto (Braatz) or Manual (skip controller, optional `xmv_schedule`).
    #[serde(default)]
    pub loop_mode: BTreeMap<usize, LoopMode>,
    #[serde(default)]
    pub mv_channels: Vec<MvChannel>,
    #[serde(default)]
    pub cv_channels: Vec<CvChannel>,
    /// When set, write `{stem}.csv` and `{stem}.meta.json` under this directory.
    #[serde(default)]
    pub export_dir: Option<PathBuf>,
    #[serde(default = "default_export_stem")]
    pub export_stem: String,
    /// If true, do not cap samples at `MAX_SAMPLES` (for disk export).
    #[serde(default)]
    pub full_record: bool,
}

fn default_export_stem() -> String {
    "te_experiment".to_string()
}

#[derive(Clone, Debug, Serialize)]
pub struct ExperimentMeta {
    pub format: &'static str,
    pub csv_time_base: &'static str,
    pub delta_t_seconds: f64,
    pub record_every: usize,
    pub seed: f64,
    pub mode: SimMode,
    pub npts: usize,
    pub steps_run: usize,
    pub mv_channels: Vec<MvChannel>,
    pub cv_channels: Vec<CvChannel>,
    pub loop_mode: BTreeMap<usize, LoopMode>,
    pub setpoint_schedule: Vec<SchedulePoint>,
    pub xmv_schedule: Vec<SchedulePoint>,
    pub injections: Vec<Injection>,
    pub analyzer_note: &'static str,
    pub shutdown: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shutdown_time_s: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shutdown_reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExperimentResult {
    #[serde(flatten)]
    pub sim: crate::simulate::SimulationResult,
    pub setpt: Vec<Vec<f32>>,
    pub mv_export: Vec<Vec<f32>>,
    pub cv_export: Vec<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_path: Option<PathBuf>,
}

impl ExperimentRequest {
    pub fn validate(&self) -> Result<usize, SimulationError> {
        self.sim.validate()?;
        if self.sim.npts == 0 || self.sim.npts > MAX_NPTS {
            return Err(SimulationError(format!(
                "npts must be 1..={MAX_NPTS}, got {}",
                self.sim.npts
            )));
        }
        for pt in self
            .setpoint_schedule
            .iter()
            .chain(self.xmv_schedule.iter())
        {
            if pt.start_step == 0 {
                return Err(SimulationError(
                    "schedule start_step must be >= 1".into(),
                ));
            }
        }
        for pt in &self.setpoint_schedule {
            if !(1..=20).contains(&pt.n) {
                return Err(SimulationError(format!(
                    "SETPT index must be 1..=20, got {}",
                    pt.n
                )));
            }
        }
        for pt in &self.xmv_schedule {
            if !(1..=N_XMV).contains(&pt.n) {
                return Err(SimulationError(format!(
                    "XMV index must be 1..={N_XMV}, got {}",
                    pt.n
                )));
            }
        }
        for ch in &self.mv_channels {
            match ch.kind {
                MvChannelKind::Setpoint if !(1..=20).contains(&ch.n) => {
                    return Err(SimulationError(format!(
                        "MV channel SETPT index must be 1..=20, got {}",
                        ch.n
                    )));
                }
                MvChannelKind::Xmv if !(1..=N_XMV).contains(&ch.n) => {
                    return Err(SimulationError(format!(
                        "MV channel XMV index must be 1..={N_XMV}, got {}",
                        ch.n
                    )));
                }
                _ => {}
            }
        }
        for ch in &self.cv_channels {
            if !(1..=N_XMEAS).contains(&ch.n) {
                return Err(SimulationError(format!(
                    "CV channel XMEAS index must be 1..=N_XMEAS, got {}",
                    ch.n
                )));
            }
        }
        let every = self.sim.record_every.max(1);
        let record_every = if self.full_record || self.export_dir.is_some() {
            every
        } else {
            effective_record_every(self.sim.npts, every)
        };
        Ok(record_every)
    }
}

/// Default reactor-temperature identification mapping: excite `TIC1009` / `XMEAS(9)`.
pub fn default_reactor_temp_mapping() -> (Vec<MvChannel>, Vec<CvChannel>) {
    (
        vec![MvChannel {
            kind: MvChannelKind::Setpoint,
            n: 18,
            tag: Some("TIC1009".into()),
        }],
        vec![CvChannel {
            n: 9,
            tag: Some("TI1009".into()),
        }],
    )
}

pub fn run(req: &ExperimentRequest) -> Result<ExperimentResult, SimulationError> {
    let record_every = req.validate()?;
    let dt = default_delta_t();
    let mut process = TennesseeEastmanProcess::with_seed(req.sim.seed);
    process.teinit();
    for i in 0..N_IDV {
        process.set_idv(i + 1, false);
    }

    let manual: BTreeSet<usize> = req
        .loop_mode
        .iter()
        .filter(|(_, m)| **m == LoopMode::Manual)
        .map(|(&n, _)| n)
        .collect();
    let mask = ControllerMask::from_manual_setpoints(&manual);

    let mut result = match req.sim.mode {
        SimMode::ClosedLoop => run_closed_experiment(req, &mut process, dt, record_every, &mask),
        SimMode::OpenLoop => run_open_experiment(req, &mut process, dt, record_every),
    }?;

    if let Some(dir) = &req.export_dir {
        let (csv_path, meta_path) = write_export(dir, &req.export_stem, req, &result)?;
        result.csv_path = Some(csv_path);
        result.meta_path = Some(meta_path);
    }

    Ok(result)
}

fn run_closed_experiment(
    req: &ExperimentRequest,
    process: &mut TennesseeEastmanProcess,
    dt: f64,
    record_every: usize,
    mask: &ControllerMask,
) -> Result<ExperimentResult, SimulationError> {
    let mut ctrl = PlantWideController::new(dt);
    PlantWideController::apply_base_xmv(process);
    let overrides: Vec<(usize, f64)> = req
        .sim
        .setpoints
        .iter()
        .map(|(&n, &v)| (n, v))
        .collect();
    ctrl.apply_setpoint_overrides(&overrides);

    let held: Vec<(usize, f64)> = req
        .sim
        .held_setpoints
        .iter()
        .map(|&n| {
            let value = req
                .sim
                .setpoints
                .get(&n)
                .copied()
                .unwrap_or(ctrl.setpt[n - 1]);
            (n, value)
        })
        .collect();

    let mut rec = ExperimentRecorder::new(req, record_every);

    apply_schedules(0, &mut ctrl, process, req);
    rec.push(0, process, &ctrl);

    for i in 1..=req.sim.npts {
        apply_injections(process, &req.sim.injections, i);
        apply_schedules(i, &mut ctrl, process, req);
        ctrl.step_masked(process, i, mask);
        apply_manual_xmv(i, process, req);
        for &(n, value) in &held {
            ctrl.setpt[n - 1] = value;
        }
        if should_record_step(i, record_every, &req.sim.injections) {
            rec.push_if_new(i as u32, process, &ctrl);
        }
        process.integrate(dt);
        ctrl.constrain_hand(process);
        if process.is_shutdown() {
            rec.note_shutdown(i as u32, process);
            rec.push_if_new(i as u32, process, &ctrl);
            break;
        }
    }

    Ok(rec.finish(req, record_every))
}

fn run_open_experiment(
    req: &ExperimentRequest,
    process: &mut TennesseeEastmanProcess,
    dt: f64,
    record_every: usize,
) -> Result<ExperimentResult, SimulationError> {
    process.set_xmv(10, 38.0);
    for (&n, &value) in &req.sim.open_loop_xmv {
        process.set_xmv(n, value);
    }
    let mut ctrl = StripperLevelController::from_process(process);
    if let Some(sp) = req.sim.open_loop_stripper_sp {
        ctrl.setpt = sp;
    }
    let setpt_rec = PlantWideController::new(dt);

    let mut rec = ExperimentRecorder::new(req, record_every);
    apply_xmv_schedules(0, process, req);
    rec.push(0, process, &setpt_rec);

    for i in 1..=req.sim.npts {
        apply_injections(process, &req.sim.injections, i);
        apply_xmv_schedules(i, process, req);
        ctrl.apply(process, dt);
        if should_record_step(i, record_every, &req.sim.injections) {
            rec.push_if_new(i as u32, process, &setpt_rec);
        }
        process.integrate(dt);
        if process.is_shutdown() {
            rec.note_shutdown(i as u32, process);
            rec.push_if_new(i as u32, process, &setpt_rec);
            break;
        }
    }

    Ok(rec.finish(req, record_every))
}

fn apply_schedules(
    step: usize,
    ctrl: &mut PlantWideController,
    process: &mut TennesseeEastmanProcess,
    req: &ExperimentRequest,
) {
    for sp in &req.setpoint_schedule {
        if step >= sp.start_step {
            ctrl.setpt[sp.n - 1] = sp.value;
        }
    }
    apply_xmv_schedules(step, process, req);
}

fn apply_xmv_schedules(step: usize, process: &mut TennesseeEastmanProcess, req: &ExperimentRequest) {
    for pt in &req.xmv_schedule {
        if step >= pt.start_step {
            process.set_xmv(pt.n, pt.value);
        }
    }
}

fn apply_manual_xmv(
    step: usize,
    process: &mut TennesseeEastmanProcess,
    req: &ExperimentRequest,
) {
    for (&setpt, mode) in &req.loop_mode {
        if *mode != LoopMode::Manual {
            continue;
        }
        if let Some(xmv) = direct_xmv_for_setpoint(setpt) {
            if let Some(v) = scheduled_xmv_at(step, xmv, req) {
                process.set_xmv(xmv, v);
            }
        }
    }
}

fn direct_xmv_for_setpoint(setpt: usize) -> Option<usize> {
    match setpt {
        1..=11 => Some(setpt),
        _ => None,
    }
}

fn scheduled_xmv_at(step: usize, xmv: usize, req: &ExperimentRequest) -> Option<f64> {
    req.xmv_schedule
        .iter()
        .filter(|p| p.n == xmv && step >= p.start_step)
        .last()
        .map(|p| p.value)
}

fn mv_value_at_sample(
    ch: &MvChannel,
    sample_idx: usize,
    rec: &ExperimentRecorder,
    ctrl_at: &[f64; 20],
) -> f32 {
    match ch.kind {
        MvChannelKind::Setpoint => ctrl_at[ch.n - 1] as f32,
        MvChannelKind::Xmv => rec.xmv[ch.n - 1][sample_idx],
    }
}

struct ExperimentRecorder {
    mode: SimMode,
    npts: usize,
    seed: f64,
    time_s: Vec<u32>,
    xmeas: Vec<Vec<f32>>,
    xmv: Vec<Vec<f32>>,
    setpt: Vec<Vec<f32>>,
    shutdown: bool,
    shutdown_time_s: Option<u32>,
    shutdown_reasons: Vec<String>,
    injections: Vec<Injection>,
    steps_run: usize,
}

impl ExperimentRecorder {
    fn new(req: &ExperimentRequest, record_every: usize) -> Self {
        let cap = req.sim.npts / record_every + 2;
        Self {
            mode: req.sim.mode,
            npts: req.sim.npts,
            seed: req.sim.seed,
            time_s: Vec::with_capacity(cap),
            xmeas: vec![Vec::with_capacity(cap); N_XMEAS],
            xmv: vec![Vec::with_capacity(cap); N_XMV],
            setpt: vec![Vec::with_capacity(cap); 20],
            shutdown: false,
            shutdown_time_s: None,
            shutdown_reasons: Vec::new(),
            injections: req.sim.injections.clone(),
            steps_run: 0,
        }
    }

    fn push(&mut self, t: u32, process: &TennesseeEastmanProcess, ctrl: &PlantWideController) {
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
        for i in 0..20 {
            self.setpt[i].push(ctrl.setpt[i] as f32);
        }
    }

    fn push_if_new(&mut self, t: u32, process: &TennesseeEastmanProcess, ctrl: &PlantWideController) {
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

    fn finish(self, req: &ExperimentRequest, record_every: usize) -> ExperimentResult {
        let steps_run = if self.shutdown {
            self.steps_run
        } else {
            req.sim.npts
        };

        let mut ctrl_at = [0.0; 20];
        let mv_export: Vec<Vec<f32>> = if req.mv_channels.is_empty() {
            Vec::new()
        } else {
            req.mv_channels
                .iter()
                .map(|ch| {
                    (0..self.time_s.len())
                        .map(|k| {
                            for i in 0..20 {
                                ctrl_at[i] = self.setpt[i][k] as f64;
                            }
                            mv_value_at_sample(ch, k, &self, &ctrl_at)
                        })
                        .collect()
                })
                .collect()
        };

        let cv_export: Vec<Vec<f32>> = req
            .cv_channels
            .iter()
            .map(|ch| {
                self.xmeas[ch.n - 1]
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect();

        let sim = crate::simulate::SimulationResult {
            mode: self.mode,
            npts: self.npts,
            steps_run,
            record_every,
            delta_t_hours: default_delta_t(),
            seed: self.seed,
            time_s: self.time_s.clone(),
            xmeas: self.xmeas.clone(),
            xmv: self.xmv.clone(),
            setpt: if self.setpt[0].is_empty() {
                None
            } else {
                Some(self.setpt.clone())
            },
            shutdown: self.shutdown,
            shutdown_time_s: self.shutdown_time_s,
            shutdown_reasons: self.shutdown_reasons.clone(),
            injections: self.injections.clone(),
        };

        ExperimentResult {
            sim,
            setpt: self.setpt,
            mv_export,
            cv_export,
            csv_path: None,
            meta_path: None,
        }
    }
}

pub fn write_export(
    dir: &Path,
    stem: &str,
    req: &ExperimentRequest,
    result: &ExperimentResult,
) -> Result<(PathBuf, PathBuf), SimulationError> {
    fs::create_dir_all(dir).map_err(|e| SimulationError(e.to_string()))?;
    let csv_path = dir.join(format!("{stem}.csv"));
    let meta_path = dir.join(format!("{stem}.meta.json"));

    let csv = mimo_sim_csv(
        &result.sim.time_s,
        &result.mv_export,
        &result.cv_export,
        req.sim.record_every.max(1),
    );
    fs::write(&csv_path, csv).map_err(|e| SimulationError(e.to_string()))?;

    let meta = ExperimentMeta {
        format: "mimo-sim-csv-v1",
        csv_time_base: CSV_TIME_BASE,
        delta_t_seconds: 1.0,
        record_every: result.sim.record_every,
        seed: req.sim.seed,
        mode: req.sim.mode,
        npts: req.sim.npts,
        steps_run: result.sim.steps_run,
        mv_channels: req.mv_channels.clone(),
        cv_channels: req.cv_channels.clone(),
        loop_mode: req.loop_mode.clone(),
        setpoint_schedule: req.setpoint_schedule.clone(),
        xmv_schedule: req.xmv_schedule.clone(),
        injections: req.sim.injections.clone(),
        analyzer_note: "XMEAS(23..41) sampled at 0.1 h or 0.25 h with dead time in TEFUNC",
        shutdown: result.sim.shutdown,
        shutdown_time_s: result.sim.shutdown_time_s,
        shutdown_reasons: result.sim.shutdown_reasons.clone(),
    };
    let meta_json =
        serde_json::to_string_pretty(&meta).map_err(|e| SimulationError(e.to_string()))?;
    fs::write(&meta_path, meta_json).map_err(|e| SimulationError(e.to_string()))?;

    Ok((csv_path, meta_path))
}

pub fn mimo_sim_csv(
    time_s: &[u32],
    mv: &[Vec<f32>],
    cv: &[Vec<f32>],
    record_every: usize,
) -> String {
    let mut headers = vec!["time".to_string()];
    for j in 0..mv.len() {
        headers.push(format!("MV{}", j + 1));
    }
    for i in 0..cv.len() {
        headers.push(format!("CV{}", i + 1));
    }

    let base_secs = parse_base_epoch(CSV_TIME_BASE);
    let mut lines = vec![headers.join(",")];
    for (k, &t) in time_s.iter().enumerate() {
        let epoch = base_secs + t as i64;
        let mut row = vec![format_epoch(epoch)];
        for col in mv {
            row.push(csv_cell(col.get(k).copied()));
        }
        for col in cv {
            row.push(csv_cell(col.get(k).copied()));
        }
        lines.push(row.join(","));
    }
    let _ = record_every;
    format!("\u{feff}{}\n", lines.join("\n"))
}

fn csv_cell(v: Option<f32>) -> String {
    match v {
        Some(x) if x.is_finite() => x.to_string(),
        _ => String::new(),
    }
}

fn parse_base_epoch(s: &str) -> i64 {
    // 2026-03-26 00:00:00 UTC as unix-ish offset from itself = 0; use simple offset from base.
    let _ = s;
    0i64
}

fn format_epoch(seconds_from_base: i64) -> String {
    // Base 2026-03-26 00:00:00 + seconds_from_base
    const BASE_Y: i64 = 2026;
    const BASE_M: i64 = 3;
    const BASE_D: i64 = 26;
    let total = seconds_from_base;
    let day_secs = total % 86_400;
    let days = total / 86_400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    let (y, mo, d) = add_days(BASE_Y, BASE_M, BASE_D, days);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
}

fn add_days(y: i64, m: i64, d: i64, add: i64) -> (i64, i64, i64) {
    let mut y = y;
    let mut m = m;
    let mut d = d + add;
    while d > days_in_month(y, m) {
        d -= days_in_month(y, m);
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    while d < 1 {
        m -= 1;
        if m < 1 {
            m = 12;
            y -= 1;
        }
        d += days_in_month(y, m);
    }
    (y, m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub fn schedules_from_signal(
    n: usize,
    is_setpoint: bool,
    base: f64,
    values: &[f64],
    record_every: usize,
) -> Vec<SchedulePoint> {
    let mut out = Vec::new();
    let mut prev = base;
    for (k, &v) in values.iter().enumerate().skip(1) {
        if (v - prev).abs() > f64::EPSILON {
            out.push(SchedulePoint {
                start_step: k * record_every,
                n,
                value: v,
            });
            prev = v;
        }
    }
    if out.is_empty() && (values.first().copied().unwrap_or(base) - base).abs() > f64::EPSILON {
        out.push(SchedulePoint {
            start_step: 1,
            n,
            value: values[0],
        });
    }
    let _ = is_setpoint;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::DEFAULT_RNG_SEED;
    use std::collections::BTreeMap;

    #[test]
    fn experiment_csv_headers() {
        let csv = mimo_sim_csv(
            &[0, 60, 120],
            &[vec![1.0, 2.0, 3.0]],
            &[vec![10.0, 11.0, 12.0]],
            60,
        );
        assert!(csv.starts_with('\u{feff}'));
        assert!(csv.contains("time,MV1,CV1"));
        assert!(csv.contains("2026-03-26"));
    }

    #[test]
    fn closed_loop_experiment_runs() {
        let (mv, cv) = default_reactor_temp_mapping();
        let req = ExperimentRequest {
            sim: SimulationRequest {
                mode: SimMode::ClosedLoop,
                npts: 600,
                record_every: 60,
                seed: DEFAULT_RNG_SEED,
                setpoints: BTreeMap::new(),
                held_setpoints: vec![18],
                injections: vec![],
                open_loop_xmv: BTreeMap::new(),
                open_loop_stripper_sp: None,
            },
            setpoint_schedule: vec![SchedulePoint {
                start_step: 300,
                n: 18,
                value: 121.0,
            }],
            xmv_schedule: vec![],
            loop_mode: BTreeMap::new(),
            mv_channels: mv,
            cv_channels: cv,
            export_dir: None,
            export_stem: "test".into(),
            full_record: true,
        };
        let out = run(&req).expect("experiment");
        assert_eq!(out.mv_export.len(), 1);
        assert_eq!(out.cv_export.len(), 1);
        assert_eq!(out.sim.time_s.len(), out.mv_export[0].len());
    }
}

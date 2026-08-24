//! Closed-loop plant-wide control from `archive/temain_mod.f` (Russell / Chiang / Braatz).

#![allow(clippy::manual_is_multiple_of, clippy::neg_multiply)]

use std::collections::BTreeSet;

use crate::process::TennesseeEastmanProcess;

/// Skip mask for `contrl1..contrl22` (index matches Fortran controller number).
#[derive(Clone, Debug, Default)]
pub struct ControllerMask {
    pub skip: [bool; 23],
}

impl ControllerMask {
    pub fn from_manual_setpoints(manual: &BTreeSet<usize>) -> Self {
        let mut skip = [false; 23];
        for &setpt in manual {
            for &c in contrls_for_setpoint(setpt) {
                if c < skip.len() {
                    skip[c] = true;
                }
            }
        }
        Self { skip }
    }
}

/// 1-based `SETPT(n)` → `contrl` indices skipped in Manual loop mode.
pub fn contrls_for_setpoint(setpt: usize) -> &'static [usize] {
    match setpt {
        1 => &[1],
        2 => &[2],
        3 => &[3],
        4 => &[4],
        5 => &[5],
        6 => &[6],
        7 => &[7],
        8 => &[8],
        9 => &[9],
        10 => &[10],
        11 => &[11],
        12 => &[22],
        13 => &[13],
        14 => &[14],
        15 => &[15],
        16 => &[16],
        17 => &[17],
        18 => &[18],
        19 => &[19],
        20 => &[20],
        _ => &[],
    }
}

#[derive(Clone, Debug)]
pub struct ClosedLoopConfig {
    pub npts: usize,
    pub sspts: usize,
    pub delta_t: f64,
    /// 1-based IDV indices enabled after `sspts` (Fortran default: `IDV(12)=1`).
    pub idv_after_ss: Vec<usize>,
}

impl Default for ClosedLoopConfig {
    fn default() -> Self {
        Self {
            npts: 172_800,
            sspts: 3600 * 8,
            delta_t: crate::process::default_delta_t(),
            idv_after_ss: vec![12],
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlantWideController {
    pub setpt: [f64; 20],
    pub delta_t: f64,
    pub flag: i32,
    gain: [f64; 23],
    taui: [f64; 23],
    errold: [f64; 23],
}

impl PlantWideController {
    pub fn new(delta_t: f64) -> Self {
        let mut this = Self {
            setpt: [0.0; 20],
            delta_t,
            flag: 0,
            gain: [0.0; 23],
            taui: [0.0; 23],
            errold: [0.0; 23],
        };
        this.setpt[0] = 3664.0;
        this.gain[1] = 1.0;
        this.setpt[1] = 4509.3;
        this.gain[2] = 1.0;
        this.setpt[2] = 0.25052;
        this.gain[3] = 1.0;
        this.setpt[3] = 9.3477;
        this.gain[4] = 1.0;
        this.setpt[4] = 26.902;
        this.gain[5] = -0.083;
        this.taui[5] = 1.0 / 3600.0;
        this.setpt[5] = 0.33712;
        this.gain[6] = 1.22;
        this.setpt[6] = 50.0;
        this.gain[7] = -2.06;
        this.setpt[7] = 50.0;
        this.gain[8] = -1.62;
        this.setpt[8] = 230.31;
        this.gain[9] = 0.41;
        this.setpt[9] = 94.599;
        this.gain[10] = -0.156 * 10.0;
        this.taui[10] = 1452.0 / 3600.0;
        this.setpt[10] = 22.949;
        this.gain[11] = 1.09;
        this.taui[11] = 2600.0 / 3600.0;
        this.setpt[12] = 32.188;
        this.gain[13] = 18.0;
        this.taui[13] = 3168.0 / 3600.0;
        this.setpt[13] = 6.8820;
        this.gain[14] = 8.3;
        this.taui[14] = 3168.0 / 3600.0;
        this.setpt[14] = 18.776;
        this.gain[15] = 2.37;
        this.taui[15] = 5069.0 / 3600.0;
        this.setpt[15] = 65.731;
        this.gain[16] = 1.69 / 10.0;
        this.taui[16] = 236.0 / 3600.0;
        this.setpt[16] = 75.000;
        this.gain[17] = 11.1 / 10.0;
        this.taui[17] = 3168.0 / 3600.0;
        this.setpt[17] = 120.40;
        this.gain[18] = 2.83 * 10.0;
        this.taui[18] = 982.0 / 3600.0;
        this.setpt[18] = 13.823;
        this.gain[19] = -83.2 / 5.0 / 3.0;
        this.taui[19] = 6336.0 / 3600.0;
        this.setpt[19] = 0.83570;
        this.gain[20] = -16.3 / 5.0;
        this.taui[20] = 12408.0 / 3600.0;
        this.setpt[11] = 2633.7;
        this.gain[22] = -1.0 * 5.0;
        this.taui[22] = 1000.0 / 3600.0;
        this
    }

    pub fn default_setpoints() -> [f64; 20] {
        Self::new(crate::process::default_delta_t()).setpt
    }

    /// 1-based `SETPT(n)` overrides from `temain_mod.f`.
    pub fn apply_setpoint_overrides(&mut self, overrides: &[(usize, f64)]) {
        for &(n, value) in overrides {
            assert!((1..=20).contains(&n), "SETPT index must be 1..=20");
            self.setpt[n - 1] = value;
        }
    }

    /// `temain_mod.f` base-case valve positions after `TEINIT`.
    pub fn apply_base_xmv(process: &mut TennesseeEastmanProcess) {
        process.set_xmv(1, 63.053);
        process.set_xmv(2, 53.980);
        process.set_xmv(3, 24.644);
        process.set_xmv(4, 61.302);
        process.set_xmv(5, 22.210);
        process.set_xmv(6, 40.064);
        process.set_xmv(7, 38.100);
        process.set_xmv(8, 46.534);
        process.set_xmv(9, 47.446);
        process.set_xmv(10, 41.106);
        process.set_xmv(11, 18.114);
    }

    /// Discrete controllers scheduled like the Fortran `MOD(I, …)` tests.
    pub fn step(&mut self, process: &mut TennesseeEastmanProcess, i: usize) {
        self.step_masked(process, i, &ControllerMask::default());
    }

    /// Like [`Self::step`] but skips controllers marked in `mask` (Manual loops).
    pub fn step_masked(
        &mut self,
        process: &mut TennesseeEastmanProcess,
        i: usize,
        mask: &ControllerMask,
    ) {
        if i % 3 == 0 {
            if !mask.skip[1] {
                self.contrl1(process);
            }
            if !mask.skip[2] {
                self.contrl2(process);
            }
            if !mask.skip[3] {
                self.contrl3(process);
            }
            if !mask.skip[4] {
                self.contrl4(process);
            }
            if !mask.skip[5] {
                self.contrl5(process);
            }
            if !mask.skip[6] {
                self.contrl6(process);
            }
            if !mask.skip[7] {
                self.contrl7(process);
            }
            if !mask.skip[8] {
                self.contrl8(process);
            }
            if !mask.skip[9] {
                self.contrl9(process);
            }
            if !mask.skip[10] {
                self.contrl10(process);
            }
            if !mask.skip[11] {
                self.contrl11(process);
            }
            if !mask.skip[16] {
                self.contrl16(process);
            }
            if !mask.skip[17] {
                self.contrl17(process);
            }
            if !mask.skip[18] {
                self.contrl18(process);
            }
        }
        if i % 360 == 0 {
            if !mask.skip[13] {
                self.contrl13(process);
            }
            if !mask.skip[14] {
                self.contrl14(process);
            }
            if !mask.skip[15] {
                self.contrl15(process);
            }
            if !mask.skip[19] {
                self.contrl19(process);
            }
        }
        if i % 900 == 0 {
            if !mask.skip[20] {
                self.contrl20(process);
            }
        }
    }

    pub fn constrain_hand(&self, process: &mut TennesseeEastmanProcess) {
        for mv in &mut process.xmv_mut()[..11] {
            *mv = mv.clamp(0.0, 100.0);
        }
    }

    fn dx_p(&mut self, idx: usize, err: f64) -> f64 {
        let dx = self.gain[idx] * (err - self.errold[idx]);
        self.errold[idx] = err;
        dx
    }

    fn dx_pi(&mut self, idx: usize, scale: f64, err: f64) -> f64 {
        let dx = self.gain[idx]
            * ((err - self.errold[idx]) + err * self.delta_t * scale / self.taui[idx]);
        self.errold[idx] = err;
        dx
    }

    fn contrl1(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[0] - p.xmeas()[1]) * 100.0 / 5811.0;
        let dx = self.dx_p(1, err);
        p.xmv_mut()[0] += dx;
    }

    fn contrl2(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[1] - p.xmeas()[2]) * 100.0 / 8354.0;
        let dx = self.dx_p(2, err);
        p.xmv_mut()[1] += dx;
    }

    fn contrl3(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[2] - p.xmeas()[0]) * 100.0 / 1.017;
        let dx = self.dx_p(3, err);
        p.xmv_mut()[2] += dx;
    }

    fn contrl4(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[3] - p.xmeas()[3]) * 100.0 / 15.25;
        let dx = self.dx_p(4, err);
        p.xmv_mut()[3] += dx;
    }

    fn contrl5(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[4] - p.xmeas()[4]) * 100.0 / 53.0;
        let dx = self.dx_pi(5, 3.0, err);
        p.xmv_mut()[4] += dx;
    }

    fn contrl6(&mut self, p: &mut TennesseeEastmanProcess) {
        let x13 = p.xmeas()[12];
        if x13 >= 2950.0 {
            p.xmv_mut()[5] = 100.0;
            self.flag = 1;
        } else if self.flag == 1 && x13 >= 2633.7 {
            p.xmv_mut()[5] = 100.0;
        } else if self.flag == 1 && x13 <= 2633.7 {
            p.xmv_mut()[5] = 40.060;
            self.setpt[5] = 0.33712;
            self.errold[6] = 0.0;
            self.flag = 0;
        } else if x13 <= 2300.0 {
            p.xmv_mut()[5] = 0.0;
            self.flag = 2;
        } else if self.flag == 2 && x13 <= 2633.7 {
            p.xmv_mut()[5] = 0.0;
        } else if self.flag == 2 && x13 >= 2633.7 {
            p.xmv_mut()[5] = 40.060;
            self.setpt[5] = 0.33712;
            self.errold[6] = 0.0;
            self.flag = 0;
        } else {
            self.flag = 0;
            let err = (self.setpt[5] - p.xmeas()[9]) * 100.0 / 1.0;
            let dx = self.dx_p(6, err);
            p.xmv_mut()[5] += dx;
        }
    }

    fn contrl7(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[6] - p.xmeas()[11]) * 100.0 / 70.0;
        let dx = self.dx_p(7, err);
        p.xmv_mut()[6] += dx;
    }

    fn contrl8(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[7] - p.xmeas()[14]) * 100.0 / 70.0;
        let dx = self.dx_p(8, err);
        p.xmv_mut()[7] += dx;
    }

    fn contrl9(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[8] - p.xmeas()[18]) * 100.0 / 460.0;
        let dx = self.dx_p(9, err);
        p.xmv_mut()[8] += dx;
    }

    fn contrl10(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[9] - p.xmeas()[20]) * 100.0 / 150.0;
        let dx = self.dx_pi(10, 3.0, err);
        p.xmv_mut()[9] += dx;
    }

    fn contrl11(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = (self.setpt[10] - p.xmeas()[16]) * 100.0 / 46.0;
        let dx = self.dx_pi(11, 3.0, err);
        p.xmv_mut()[10] += dx;
    }

    fn contrl13(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[12] - p.xmeas()[22]) * 100.0 / 100.0;
        let dx = self.dx_pi(13, 360.0, err);
        self.setpt[2] += dx * 1.017 / 100.0;
    }

    fn contrl14(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[13] - p.xmeas()[25]) * 100.0 / 100.0;
        let dx = self.dx_pi(14, 360.0, err);
        self.setpt[0] += dx * 5811.0 / 100.0;
    }

    fn contrl15(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[14] - p.xmeas()[26]) * 100.0 / 100.0;
        let dx = self.dx_pi(15, 360.0, err);
        self.setpt[1] += dx * 8354.0 / 100.0;
    }

    fn contrl16(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[15] - p.xmeas()[17]) * 100.0 / 130.0;
        let dx = self.dx_pi(16, 3.0, err);
        self.setpt[8] += dx * 460.0 / 100.0;
    }

    fn contrl17(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[16] - p.xmeas()[7]) * 100.0 / 50.0;
        let dx = self.dx_pi(17, 3.0, err);
        self.setpt[3] += dx * 15.25 / 100.0;
    }

    fn contrl18(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[17] - p.xmeas()[8]) * 100.0 / 150.0;
        let dx = self.dx_pi(18, 3.0, err);
        self.setpt[9] += dx * 150.0 / 100.0;
    }

    fn contrl19(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[18] - p.xmeas()[29]) * 100.0 / 26.0;
        let dx = self.dx_pi(19, 360.0, err);
        self.setpt[5] += dx * 1.0 / 100.0;
    }

    fn contrl20(&mut self, p: &TennesseeEastmanProcess) {
        let err = (self.setpt[19] - p.xmeas()[37]) * 100.0 / 1.6;
        let dx = self.dx_pi(20, 900.0, err);
        self.setpt[15] += dx * 130.0 / 100.0;
    }

    /// Present in `temain_mod.f` but not called from the main loop.
    pub fn contrl22(&mut self, p: &mut TennesseeEastmanProcess) {
        let err = self.setpt[11] - p.xmeas()[12];
        let dx = self.dx_pi(22, 3.0, err);
        p.xmv_mut()[5] += dx;
    }
}

//! Open-loop demonstration from `temain.f`: one stripper-level PI loop.

use crate::process::TennesseeEastmanProcess;

#[derive(Clone, Debug)]
pub struct StripperLevelController {
    pub setpt: f64,
    pub gain: f64,
    pub taui: f64,
    pub errold: f64,
}

impl StripperLevelController {
    /// `SETPT = XMEAS(15) + 15`, `GAIN = 2`, `TAUI = 5 min`.
    pub fn from_process(process: &TennesseeEastmanProcess) -> Self {
        Self {
            setpt: process.xmeas()[14] + 15.0,
            gain: 2.0,
            taui: 5.0,
            errold: 0.0,
        }
    }

    /// Velocity-form PI on stripper level → `XMV(8)`.
    pub fn apply(&mut self, process: &mut TennesseeEastmanProcess, dt: f64) {
        let err = self.setpt - process.xmeas()[14];
        let dxmv = self.gain * ((err - self.errold) + err * dt * 60.0 / self.taui);
        process.xmv_mut()[7] -= dxmv;
        self.errold = err;
    }
}

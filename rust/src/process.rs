//! Process model translated from `teprob.f`: `TEINIT`, `TEFUNC`, `TESUB1`–`TESUB8`.
//!
//! Arithmetic is IEEE-754 double throughout. Constants are taken as written in
//! the Fortran source rather than rounded through default-kind `REAL`.

pub const N_STATES: usize = 50;
pub const N_XMEAS: usize = 41;
pub const N_XMV: usize = 12;
pub const N_IDV: usize = 20;
pub const N_COMPONENTS: usize = 8;
pub const N_STREAMS: usize = 13;
pub const OBSERVATION_LEN: usize = N_XMEAS + 11;
pub const DEFAULT_RNG_SEED: f64 = 4_651_207_995.0;

/// Integrator step of 1 second, expressed in hours.
pub fn default_delta_t() -> f64 {
    1.0 / 3600.0
}

#[derive(Clone, Debug)]
pub struct TennesseeEastmanProcess {
    pub time: f64,
    pub yy: [f64; N_STATES],
    pub yp: [f64; N_STATES],
    xmeas: [f64; N_XMEAS],
    xmv: [f64; N_XMV],
    idv: [i32; N_IDV],
    pub g: f64,
    pub isd: i32,

    xmw: [f64; 8],
    avp: [f64; 8],
    bvp: [f64; 8],
    cvp: [f64; 8],
    ad: [f64; 8],
    bd: [f64; 8],
    cd: [f64; 8],
    ah: [f64; 8],
    bh: [f64; 8],
    ch: [f64; 8],
    av: [f64; 8],
    ag: [f64; 8],
    bg: [f64; 8],
    cg: [f64; 8],

    uclr: [f64; 8],
    ucvr: [f64; 8],
    utlr: f64,
    utvr: f64,
    xlr: [f64; 8],
    xvr: [f64; 8],
    etr: f64,
    esr: f64,
    tcr: f64,
    tkr: f64,
    dlr: f64,
    vlr: f64,
    vvr: f64,
    vtr: f64,
    ptr: f64,
    ppr: [f64; 8],
    crxr: [f64; 8],
    rr: [f64; 4],
    rh: f64,
    fwr: f64,
    twr: f64,
    qur: f64,
    hwr: f64,
    uar: f64,

    ucls: [f64; 8],
    ucvs: [f64; 8],
    utls: f64,
    utvs: f64,
    xls: [f64; 8],
    xvs: [f64; 8],
    ets: f64,
    ess: f64,
    tcs: f64,
    tks: f64,
    dls: f64,
    vls: f64,
    vvs: f64,
    vts: f64,
    pts: f64,
    pps: [f64; 8],
    fws: f64,
    tws: f64,
    qus: f64,
    hws: f64,

    uclc: [f64; 8],
    utlc: f64,
    xlc: [f64; 8],
    etc: f64,
    esc: f64,
    tcc: f64,
    dlc: f64,
    vlc: f64,
    vtc: f64,
    quc: f64,

    ucvv: [f64; 8],
    utvv: f64,
    xvv: [f64; 8],
    etv: f64,
    esv: f64,
    tcv: f64,
    tkv: f64,
    vtv: f64,
    ptv: f64,

    vcv: [f64; 12],
    vrng: [f64; 12],
    vtau: [f64; 12],
    ftm: [f64; 13],
    fcm: [[f64; 8]; 13],
    xst: [[f64; 8]; 13],
    xmws: [f64; 13],
    hst: [f64; 13],
    tst: [f64; 13],
    sfr: [f64; 8],
    cpflmx: f64,
    cpprmx: f64,
    cpdh: f64,
    tcwr: f64,
    tcws: f64,
    htr: [f64; 3],
    agsp: f64,
    xdel: [f64; 41],
    xns: [f64; 41],
    tgas: f64,
    tprod: f64,
    vst: [f64; 12],
    ivst: [i32; 12],

    adist: [f64; 12],
    bdist: [f64; 12],
    cdist: [f64; 12],
    ddist: [f64; 12],
    tlast: [f64; 12],
    tnext: [f64; 12],
    hspan: [f64; 12],
    hzero: [f64; 12],
    sspan: [f64; 12],
    szero: [f64; 12],
    spspan: [f64; 12],
    idvwlk: [i32; 12],
}

impl Default for TennesseeEastmanProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl TennesseeEastmanProcess {
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_RNG_SEED)
    }

    pub fn with_seed(g: f64) -> Self {
        Self {
            time: 0.0,
            yy: [0.0; N_STATES],
            yp: [0.0; N_STATES],
            xmeas: [0.0; N_XMEAS],
            xmv: [0.0; N_XMV],
            idv: [0; N_IDV],
            g,
            isd: 0,
            xmw: [0.0; 8],
            avp: [0.0; 8],
            bvp: [0.0; 8],
            cvp: [0.0; 8],
            ad: [0.0; 8],
            bd: [0.0; 8],
            cd: [0.0; 8],
            ah: [0.0; 8],
            bh: [0.0; 8],
            ch: [0.0; 8],
            av: [0.0; 8],
            ag: [0.0; 8],
            bg: [0.0; 8],
            cg: [0.0; 8],
            uclr: [0.0; 8],
            ucvr: [0.0; 8],
            utlr: 0.0,
            utvr: 0.0,
            xlr: [0.0; 8],
            xvr: [0.0; 8],
            etr: 0.0,
            esr: 0.0,
            tcr: 0.0,
            tkr: 0.0,
            dlr: 0.0,
            vlr: 0.0,
            vvr: 0.0,
            vtr: 0.0,
            ptr: 0.0,
            ppr: [0.0; 8],
            crxr: [0.0; 8],
            rr: [0.0; 4],
            rh: 0.0,
            fwr: 0.0,
            twr: 0.0,
            qur: 0.0,
            hwr: 0.0,
            uar: 0.0,
            ucls: [0.0; 8],
            ucvs: [0.0; 8],
            utls: 0.0,
            utvs: 0.0,
            xls: [0.0; 8],
            xvs: [0.0; 8],
            ets: 0.0,
            ess: 0.0,
            tcs: 0.0,
            tks: 0.0,
            dls: 0.0,
            vls: 0.0,
            vvs: 0.0,
            vts: 0.0,
            pts: 0.0,
            pps: [0.0; 8],
            fws: 0.0,
            tws: 0.0,
            qus: 0.0,
            hws: 0.0,
            uclc: [0.0; 8],
            utlc: 0.0,
            xlc: [0.0; 8],
            etc: 0.0,
            esc: 0.0,
            tcc: 0.0,
            dlc: 0.0,
            vlc: 0.0,
            vtc: 0.0,
            quc: 0.0,
            ucvv: [0.0; 8],
            utvv: 0.0,
            xvv: [0.0; 8],
            etv: 0.0,
            esv: 0.0,
            tcv: 0.0,
            tkv: 0.0,
            vtv: 0.0,
            ptv: 0.0,
            vcv: [0.0; 12],
            vrng: [0.0; 12],
            vtau: [0.0; 12],
            ftm: [0.0; 13],
            fcm: [[0.0; 8]; 13],
            xst: [[0.0; 8]; 13],
            xmws: [0.0; 13],
            hst: [0.0; 13],
            tst: [0.0; 13],
            sfr: [0.0; 8],
            cpflmx: 0.0,
            cpprmx: 0.0,
            cpdh: 0.0,
            tcwr: 0.0,
            tcws: 0.0,
            htr: [0.0; 3],
            agsp: 0.0,
            xdel: [0.0; 41],
            xns: [0.0; 41],
            tgas: 0.0,
            tprod: 0.0,
            vst: [0.0; 12],
            ivst: [0; 12],
            adist: [0.0; 12],
            bdist: [0.0; 12],
            cdist: [0.0; 12],
            ddist: [0.0; 12],
            tlast: [0.0; 12],
            tnext: [0.0; 12],
            hspan: [0.0; 12],
            hzero: [0.0; 12],
            sspan: [0.0; 12],
            szero: [0.0; 12],
            spspan: [0.0; 12],
            idvwlk: [0; 12],
        }
    }

    pub fn xmeas(&self) -> &[f64; N_XMEAS] {
        &self.xmeas
    }

    pub fn xmv(&self) -> &[f64; N_XMV] {
        &self.xmv
    }

    pub fn xmv_mut(&mut self) -> &mut [f64; N_XMV] {
        &mut self.xmv
    }

    pub fn idv(&self) -> &[i32; N_IDV] {
        &self.idv
    }

    /// 1-based `IDV(n)`.
    pub fn set_idv(&mut self, n: usize, on: bool) {
        assert!((1..=N_IDV).contains(&n), "IDV index must be 1..=20");
        self.idv[n - 1] = i32::from(on);
    }

    /// 1-based `XMV(n)`.
    pub fn set_xmv(&mut self, n: usize, value: f64) {
        assert!((1..=N_XMV).contains(&n), "XMV index must be 1..=12");
        self.xmv[n - 1] = value;
    }

    pub fn is_shutdown(&self) -> bool {
        self.isd != 0
    }

    /// Interlock trips from `TEFUNC` (`ISD`). Empty when the plant is running.
    pub fn shutdown_reasons(&self) -> Vec<&'static str> {
        let mut reasons = Vec::new();
        if self.xmeas[6] > 3000.0 {
            reasons.push("Reactor pressure exceeds 3000 kPa gauge");
        }
        if self.vlr / 35.3145 > 24.0 {
            reasons.push("Reactor liquid volume high");
        }
        if self.vlr / 35.3145 < 2.0 {
            reasons.push("Reactor liquid volume low");
        }
        if self.xmeas[8] > 175.0 {
            reasons.push("Reactor temperature exceeds 175 °C");
        }
        if self.vls / 35.3145 > 12.0 {
            reasons.push("Separator liquid volume high");
        }
        if self.vls / 35.3145 < 1.0 {
            reasons.push("Separator liquid volume low");
        }
        if self.vlc / 35.3145 > 8.0 {
            reasons.push("Stripper liquid volume high");
        }
        if self.vlc / 35.3145 < 1.0 {
            reasons.push("Stripper liquid volume low");
        }
        reasons
    }

    /// Observation vector used by the training/testing `.dat` files:
    /// `[XMEAS(1..41), XMV(1..11)]`.
    pub fn observation(&self) -> [f64; OBSERVATION_LEN] {
        let mut out = [0.0; OBSERVATION_LEN];
        out[..N_XMEAS].copy_from_slice(&self.xmeas);
        out[N_XMEAS..].copy_from_slice(&self.xmv[..11]);
        out
    }

    /// `TEINIT` then one `TEFUNC` at `TIME = 0`.
    pub fn teinit(&mut self) {
        self.xmw = [2.0, 25.4, 28.0, 32.0, 46.0, 48.0, 62.0, 76.0];
        self.avp = [0.0, 0.0, 0.0, 15.92, 16.35, 16.35, 16.43, 17.21];
        self.bvp = [0.0, 0.0, 0.0, -1444.0, -2114.0, -2114.0, -2748.0, -3318.0];
        self.cvp = [0.0, 0.0, 0.0, 259.0, 265.5, 265.5, 232.9, 249.6];
        self.ad = [1.0, 1.0, 1.0, 23.3, 33.9, 32.8, 49.9, 50.5];
        self.bd = [0.0, 0.0, 0.0, -0.0700, -0.0957, -0.0995, -0.0191, -0.0541];
        self.cd = [
            0.0, 0.0, 0.0, -0.0002, -0.000152, -0.000233, -0.000425, -0.000150,
        ];
        self.ah = [
            1.0e-6, 1.0e-6, 1.0e-6, 0.960e-6, 0.573e-6, 0.652e-6, 0.515e-6, 0.471e-6,
        ];
        self.bh = [0.0, 0.0, 0.0, 8.70e-9, 2.41e-9, 2.18e-9, 5.65e-10, 8.70e-10];
        self.ch = [
            0.0, 0.0, 0.0, 4.81e-11, 1.82e-11, 1.94e-11, 3.82e-12, 2.62e-12,
        ];
        self.av = [
            1.0e-6, 1.0e-6, 1.0e-6, 86.7e-6, 160.0e-6, 160.0e-6, 225.0e-6, 209.0e-6,
        ];
        self.ag = [
            3.411e-6, 0.3799e-6, 0.2491e-6, 0.3567e-6, 0.3463e-6, 0.3930e-6, 0.170e-6, 0.150e-6,
        ];
        self.bg = [
            7.18e-10, 1.08e-9, 1.36e-11, 8.51e-10, 8.96e-10, 1.02e-9, 0.0, 0.0,
        ];
        self.cg = [
            6.0e-13, -3.98e-13, -3.93e-14, -3.12e-13, -3.27e-13, -3.12e-13, 0.0, 0.0,
        ];

        self.yy = [
            10.40491389,
            4.363996017,
            7.570059737,
            0.4230042431,
            24.15513437,
            2.942597645,
            154.3770655,
            159.1865960,
            2.808522723,
            63.75581199,
            26.74026066,
            46.38532432,
            0.2464521543,
            15.20484404,
            1.852266172,
            52.44639459,
            41.20394008,
            0.5699317760,
            0.4306056376,
            7.9906200783e-3,
            0.9056036089,
            1.6054258216e-2,
            0.7509759687,
            8.8582855955e-2,
            48.27726193,
            39.38459028,
            0.3755297257,
            107.7562698,
            29.77250546,
            88.32481135,
            23.03929507,
            62.85848794,
            5.546318688,
            11.92244772,
            5.555448243,
            0.9218489762,
            94.59927549,
            77.29698353,
            63.05263039,
            53.97970677,
            24.64355755,
            61.30192144,
            22.21000000,
            40.06374673,
            38.10034370,
            46.53415582,
            47.44573456,
            41.10581288,
            18.11349055,
            50.00000000,
        ];

        for i in 0..12 {
            self.xmv[i] = self.yy[i + 38];
            self.vcv[i] = self.xmv[i];
            self.vst[i] = 2.0;
            self.ivst[i] = 0;
        }

        self.vrng[0] = 400.00;
        self.vrng[1] = 400.00;
        self.vrng[2] = 100.00;
        self.vrng[3] = 1500.00;
        self.vrng[6] = 1500.00;
        self.vrng[7] = 1000.00;
        self.vrng[8] = 0.03;
        self.vrng[9] = 1000.;
        self.vrng[10] = 1200.0;

        self.vtr = 1300.0;
        self.vts = 3500.0;
        self.vtc = 156.5;
        self.vtv = 5000.0;
        self.htr[0] = 0.06899381054;
        self.htr[1] = 0.05;
        self.hwr = 7060.;
        self.hws = 11138.;
        self.sfr = [
            0.99500, 0.99100, 0.99000, 0.91600, 0.93600, 0.93800, 5.80000e-2, 3.01000e-2,
        ];

        self.xst[0] = [0.0, 0.0001, 0.0, 0.9999, 0.0, 0.0, 0.0, 0.0];
        self.tst[0] = 45.;
        self.xst[1] = [0.0, 0.0, 0.0, 0.0, 0.9999, 0.0001, 0.0, 0.0];
        self.tst[1] = 45.;
        self.xst[2] = [0.9999, 0.0001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        self.tst[2] = 45.;
        self.xst[3] = [0.4850, 0.0050, 0.5100, 0.0, 0.0, 0.0, 0.0, 0.0];
        self.tst[3] = 45.;

        self.cpflmx = 280275.;
        self.cpprmx = 1.3;
        self.vtau = [8., 8., 6., 9., 7., 5., 5., 5., 120., 5., 5., 5.];
        for v in &mut self.vtau {
            *v /= 3600.0;
        }

        self.xns = [
            0.0012, 18.000, 22.000, 0.0500, 0.2000, 0.2100, 0.3000, 0.5000, 0.0100, 0.0017, 0.0100,
            1.0000, 0.3000, 0.1250, 1.0000, 0.3000, 0.1150, 0.0100, 1.1500, 0.2000, 0.0100, 0.0100,
            0.250, 0.100, 0.250, 0.100, 0.250, 0.025, 0.250, 0.100, 0.250, 0.100, 0.250, 0.025,
            0.050, 0.050, 0.010, 0.010, 0.010, 0.500, 0.500,
        ];
        self.idv = [0; N_IDV];

        self.hspan = [
            0.2, 0.7, 0.25, 0.7, 0.15, 0.15, 1.0, 1.0, 0.4, 1.5, 2.0, 1.5,
        ];
        self.hzero = [0.5, 1.0, 0.5, 1.0, 0.25, 0.25, 2.0, 2.0, 0.5, 2.0, 3.0, 2.0];
        self.sspan = [
            0.03, 0.003, 10.0, 10.0, 10.0, 10.0, 0.25, 0.25, 0.25, 0.0, 0.0, 0.0,
        ];
        self.szero = [
            0.485, 0.005, 45.0, 45.0, 35.0, 40.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
        ];
        self.spspan = [0.0; 12];
        for i in 0..12 {
            self.tlast[i] = 0.0;
            self.tnext[i] = 0.1;
            self.adist[i] = self.szero[i];
            self.bdist[i] = 0.0;
            self.cdist[i] = 0.0;
            self.ddist[i] = 0.0;
        }

        self.time = 0.0;
        self.tefunc();
    }

    /// Euler step from `INTGTR`: evaluate derivatives, advance `TIME`, then `YY`.
    pub fn integrate(&mut self, dt: f64) {
        self.tefunc();
        self.time += dt;
        for i in 0..N_STATES {
            self.yy[i] += self.yp[i] * dt;
        }
    }

    /// `TEFUNC` — function evaluator called by the integrator.
    pub fn tefunc(&mut self) {
        for flag in &mut self.idv {
            *flag = i32::from(*flag > 0);
        }
        self.idvwlk[0] = self.idv[7];
        self.idvwlk[1] = self.idv[7];
        self.idvwlk[2] = self.idv[8];
        self.idvwlk[3] = self.idv[9];
        self.idvwlk[4] = self.idv[10];
        self.idvwlk[5] = self.idv[11];
        self.idvwlk[6] = self.idv[12];
        self.idvwlk[7] = self.idv[12];
        self.idvwlk[8] = self.idv[15];
        self.idvwlk[9] = self.idv[16];
        self.idvwlk[10] = self.idv[17];
        self.idvwlk[11] = self.idv[19];

        for i in 0..9 {
            if self.time >= self.tnext[i] {
                let hwlk = self.tnext[i] - self.tlast[i];
                let swlk = self.adist[i]
                    + hwlk * (self.bdist[i] + hwlk * (self.cdist[i] + hwlk * self.ddist[i]));
                let spwlk =
                    self.bdist[i] + hwlk * (2.0 * self.cdist[i] + 3.0 * hwlk * self.ddist[i]);
                self.tlast[i] = self.tnext[i];
                let (a, b, c, d, tnext) = self.tesub5(
                    swlk,
                    spwlk,
                    self.tlast[i],
                    self.hspan[i],
                    self.hzero[i],
                    self.sspan[i],
                    self.szero[i],
                    self.spspan[i],
                    self.idvwlk[i],
                );
                self.adist[i] = a;
                self.bdist[i] = b;
                self.cdist[i] = c;
                self.ddist[i] = d;
                self.tnext[i] = tnext;
            }
        }
        for i in 9..12 {
            if self.time >= self.tnext[i] {
                let hwlk = self.tnext[i] - self.tlast[i];
                let swlk = self.adist[i]
                    + hwlk * (self.bdist[i] + hwlk * (self.cdist[i] + hwlk * self.ddist[i]));
                let spwlk =
                    self.bdist[i] + hwlk * (2.0 * self.cdist[i] + 3.0 * hwlk * self.ddist[i]);
                self.tlast[i] = self.tnext[i];
                if swlk > 0.1 {
                    self.adist[i] = swlk;
                    self.bdist[i] = spwlk;
                    self.cdist[i] = -(3.0 * swlk + 0.2 * spwlk) / 0.01;
                    self.ddist[i] = (2.0 * swlk + 0.1 * spwlk) / 0.001;
                    self.tnext[i] = self.tlast[i] + 0.1;
                } else {
                    let isd = -1;
                    let hwlk = self.hspan[i] * self.tesub7(isd) + self.hzero[i];
                    self.adist[i] = 0.0;
                    self.bdist[i] = 0.0;
                    self.cdist[i] = f64::from(self.idvwlk[i]) / (hwlk * hwlk);
                    self.ddist[i] = 0.0;
                    self.tnext[i] = self.tlast[i] + hwlk;
                }
            }
        }
        if self.time == 0.0 {
            for i in 0..12 {
                self.adist[i] = self.szero[i];
                self.bdist[i] = 0.0;
                self.cdist[i] = 0.0;
                self.ddist[i] = 0.0;
                self.tlast[i] = 0.0;
                self.tnext[i] = 0.1;
            }
        }

        let time = self.time;
        self.xst[3][0] = self.tesub8(0, time)
            - f64::from(self.idv[0]) * 0.03
            - f64::from(self.idv[1]) * 2.43719e-3;
        self.xst[3][1] = self.tesub8(1, time) + f64::from(self.idv[1]) * 0.005;
        self.xst[3][2] = 1.0 - self.xst[3][0] - self.xst[3][1];
        self.tst[0] = self.tesub8(2, time) + f64::from(self.idv[2]) * 5.0;
        self.tst[3] = self.tesub8(3, time);
        self.tcwr = self.tesub8(4, time) + f64::from(self.idv[3]) * 5.0;
        self.tcws = self.tesub8(5, time) + f64::from(self.idv[4]) * 5.0;
        let mut r1f = self.tesub8(6, time);
        let mut r2f = self.tesub8(7, time);

        for i in 0..3 {
            self.ucvr[i] = self.yy[i];
            self.ucvs[i] = self.yy[i + 9];
            self.uclr[i] = 0.0;
            self.ucls[i] = 0.0;
        }
        for i in 3..8 {
            self.uclr[i] = self.yy[i];
            self.ucls[i] = self.yy[i + 9];
        }
        for i in 0..8 {
            self.uclc[i] = self.yy[i + 18];
            self.ucvv[i] = self.yy[i + 27];
        }
        self.etr = self.yy[8];
        self.ets = self.yy[17];
        self.etc = self.yy[26];
        self.etv = self.yy[35];
        self.twr = self.yy[36];
        self.tws = self.yy[37];
        let mut vpos = [0.0; 12];
        for i in 0..12 {
            vpos[i] = self.yy[i + 38];
        }

        self.utlr = 0.0;
        self.utls = 0.0;
        self.utlc = 0.0;
        self.utvv = 0.0;
        for i in 0..8 {
            self.utlr += self.uclr[i];
            self.utls += self.ucls[i];
            self.utlc += self.uclc[i];
            self.utvv += self.ucvv[i];
        }
        for i in 0..8 {
            self.xlr[i] = self.uclr[i] / self.utlr;
            self.xls[i] = self.ucls[i] / self.utls;
            self.xlc[i] = self.uclc[i] / self.utlc;
            self.xvv[i] = self.ucvv[i] / self.utvv;
        }
        self.esr = self.etr / self.utlr;
        self.ess = self.ets / self.utls;
        self.esc = self.etc / self.utlc;
        self.esv = self.etv / self.utvv;

        let z = self.xlr;
        let mut t = self.tcr;
        self.tesub2(&z, &mut t, self.esr, 0);
        self.tcr = t;
        self.tkr = self.tcr + 273.15;

        let z = self.xls;
        let mut t = self.tcs;
        self.tesub2(&z, &mut t, self.ess, 0);
        self.tcs = t;
        self.tks = self.tcs + 273.15;

        let z = self.xlc;
        let mut t = self.tcc;
        self.tesub2(&z, &mut t, self.esc, 0);
        self.tcc = t;

        let z = self.xvv;
        let mut t = self.tcv;
        self.tesub2(&z, &mut t, self.esv, 2);
        self.tcv = t;
        self.tkv = self.tcv + 273.15;

        self.dlr = self.tesub4(&self.xlr, self.tcr);
        self.dls = self.tesub4(&self.xls, self.tcs);
        self.dlc = self.tesub4(&self.xlc, self.tcc);
        self.vlr = self.utlr / self.dlr;
        self.vls = self.utls / self.dls;
        self.vlc = self.utlc / self.dlc;
        self.vvr = self.vtr - self.vlr;
        self.vvs = self.vts - self.vls;

        let rg = 998.9;
        self.ptr = 0.0;
        self.pts = 0.0;
        for i in 0..3 {
            self.ppr[i] = self.ucvr[i] * rg * self.tkr / self.vvr;
            self.ptr += self.ppr[i];
            self.pps[i] = self.ucvs[i] * rg * self.tks / self.vvs;
            self.pts += self.pps[i];
        }
        for i in 3..8 {
            let vpr = (self.avp[i] + self.bvp[i] / (self.tcr + self.cvp[i])).exp();
            self.ppr[i] = vpr * self.xlr[i];
            self.ptr += self.ppr[i];
            let vpr = (self.avp[i] + self.bvp[i] / (self.tcs + self.cvp[i])).exp();
            self.pps[i] = vpr * self.xls[i];
            self.pts += self.pps[i];
        }
        self.ptv = self.utvv * rg * self.tkv / self.vtv;
        for i in 0..8 {
            self.xvr[i] = self.ppr[i] / self.ptr;
            self.xvs[i] = self.pps[i] / self.pts;
        }
        self.utvr = self.ptr * self.vvr / rg / self.tkr;
        self.utvs = self.pts * self.vvs / rg / self.tks;
        for i in 3..8 {
            self.ucvr[i] = self.utvr * self.xvr[i];
            self.ucvs[i] = self.utvs * self.xvs[i];
        }

        self.rr[0] = (31.5859536 - 40000.0 / 1.987 / self.tkr).exp() * r1f;
        self.rr[1] = (3.00094014 - 20000.0 / 1.987 / self.tkr).exp() * r2f;
        self.rr[2] = (53.4060443 - 60000.0 / 1.987 / self.tkr).exp();
        self.rr[3] = self.rr[2] * 0.767488334;
        if self.ppr[0] > 0.0 && self.ppr[2] > 0.0 {
            r1f = self.ppr[0].powf(1.1544);
            r2f = self.ppr[2].powf(0.3735);
            self.rr[0] *= r1f * r2f * self.ppr[3];
            self.rr[1] *= r1f * r2f * self.ppr[4];
        } else {
            self.rr[0] = 0.0;
            self.rr[1] = 0.0;
        }
        self.rr[2] *= self.ppr[0] * self.ppr[4];
        self.rr[3] *= self.ppr[0] * self.ppr[3];
        for r in &mut self.rr {
            *r *= self.vvr;
        }
        self.crxr[0] = -self.rr[0] - self.rr[1] - self.rr[2];
        self.crxr[2] = -self.rr[0] - self.rr[1];
        self.crxr[3] = -self.rr[0] - 1.5 * self.rr[3];
        self.crxr[4] = -self.rr[1] - self.rr[2];
        self.crxr[5] = self.rr[2] + self.rr[3];
        self.crxr[6] = self.rr[0];
        self.crxr[7] = self.rr[1];
        self.rh = self.rr[0] * self.htr[0] + self.rr[1] * self.htr[1];

        self.xmws[0] = 0.0;
        self.xmws[1] = 0.0;
        self.xmws[5] = 0.0;
        self.xmws[7] = 0.0;
        self.xmws[8] = 0.0;
        self.xmws[9] = 0.0;
        for i in 0..8 {
            self.xst[5][i] = self.xvv[i];
            self.xst[7][i] = self.xvr[i];
            self.xst[8][i] = self.xvs[i];
            self.xst[9][i] = self.xvs[i];
            self.xst[10][i] = self.xls[i];
            self.xst[12][i] = self.xlc[i];
            self.xmws[0] += self.xst[0][i] * self.xmw[i];
            self.xmws[1] += self.xst[1][i] * self.xmw[i];
            self.xmws[5] += self.xst[5][i] * self.xmw[i];
            self.xmws[7] += self.xst[7][i] * self.xmw[i];
            self.xmws[8] += self.xst[8][i] * self.xmw[i];
            self.xmws[9] += self.xst[9][i] * self.xmw[i];
        }
        self.tst[5] = self.tcv;
        self.tst[7] = self.tcr;
        self.tst[8] = self.tcs;
        self.tst[9] = self.tcs;
        self.tst[10] = self.tcs;
        self.tst[12] = self.tcc;

        self.hst[0] = self.tesub1(&self.xst[0], self.tst[0], 1);
        self.hst[1] = self.tesub1(&self.xst[1], self.tst[1], 1);
        self.hst[2] = self.tesub1(&self.xst[2], self.tst[2], 1);
        self.hst[3] = self.tesub1(&self.xst[3], self.tst[3], 1);
        self.hst[5] = self.tesub1(&self.xst[5], self.tst[5], 1);
        self.hst[7] = self.tesub1(&self.xst[7], self.tst[7], 1);
        self.hst[8] = self.tesub1(&self.xst[8], self.tst[8], 1);
        self.hst[9] = self.hst[8];
        self.hst[10] = self.tesub1(&self.xst[10], self.tst[10], 0);
        self.hst[12] = self.tesub1(&self.xst[12], self.tst[12], 0);

        self.ftm[0] = vpos[0] * self.vrng[0] / 100.0;
        self.ftm[1] = vpos[1] * self.vrng[1] / 100.0;
        self.ftm[2] = vpos[2] * (1.0 - f64::from(self.idv[5])) * self.vrng[2] / 100.0;
        self.ftm[3] =
            vpos[3] * (1.0 - f64::from(self.idv[6]) * 0.2) * self.vrng[3] / 100.0 + 1.0e-10;
        self.ftm[10] = vpos[6] * self.vrng[6] / 100.0;
        self.ftm[12] = vpos[7] * self.vrng[7] / 100.0;
        let uac = vpos[8] * self.vrng[8] * (1.0 + self.tesub8(8, time)) / 100.0;
        self.fwr = vpos[9] * self.vrng[9] / 100.0;
        self.fws = vpos[10] * self.vrng[10] / 100.0;
        self.agsp = (vpos[11] + 150.0) / 100.0;

        let mut dlp = (self.ptv - self.ptr).max(0.0);
        let mut flms = 1937.6 * dlp.sqrt();
        self.ftm[5] = flms / self.xmws[5];
        dlp = (self.ptr - self.pts).max(0.0);
        flms = 4574.21 * dlp.sqrt() * (1.0 - 0.25 * self.tesub8(11, time));
        self.ftm[7] = flms / self.xmws[7];
        dlp = (self.pts - 760.0).max(0.0);
        flms = vpos[5] * 0.151169 * dlp.sqrt();
        self.ftm[9] = flms / self.xmws[9];

        let mut pr = self.ptv / self.pts;
        if pr < 1.0 {
            pr = 1.0;
        }
        if pr > self.cpprmx {
            pr = self.cpprmx;
        }
        let flcoef = self.cpflmx / 1.197;
        flms = self.cpflmx + flcoef * (1.0 - pr.powi(3));
        self.cpdh = flms * (self.tcs + 273.15) * 1.8e-6 * 1.9872 * (self.ptv - self.pts)
            / (self.xmws[8] * self.pts);
        dlp = (self.ptv - self.pts).max(0.0);
        flms -= vpos[4] * 53.349 * dlp.sqrt();
        if flms < 1.0e-3 {
            flms = 1.0e-3;
        }
        self.ftm[8] = flms / self.xmws[8];
        self.hst[8] += self.cpdh / self.ftm[8];

        for i in 0..8 {
            self.fcm[0][i] = self.xst[0][i] * self.ftm[0];
            self.fcm[1][i] = self.xst[1][i] * self.ftm[1];
            self.fcm[2][i] = self.xst[2][i] * self.ftm[2];
            self.fcm[3][i] = self.xst[3][i] * self.ftm[3];
            self.fcm[5][i] = self.xst[5][i] * self.ftm[5];
            self.fcm[7][i] = self.xst[7][i] * self.ftm[7];
            self.fcm[8][i] = self.xst[8][i] * self.ftm[8];
            self.fcm[9][i] = self.xst[9][i] * self.ftm[9];
            self.fcm[10][i] = self.xst[10][i] * self.ftm[10];
            self.fcm[12][i] = self.xst[12][i] * self.ftm[12];
        }

        if self.ftm[10] > 0.1 {
            let tmpfac = if self.tcc > 170.0 {
                self.tcc - 120.262
            } else if self.tcc < 5.292 {
                0.1
            } else {
                363.744 / (177. - self.tcc) - 2.22579488
            };
            let vovrl = self.ftm[3] / self.ftm[10] * tmpfac;
            self.sfr[3] = 8.5010 * vovrl / (1.0 + 8.5010 * vovrl);
            self.sfr[4] = 11.402 * vovrl / (1.0 + 11.402 * vovrl);
            self.sfr[5] = 11.795 * vovrl / (1.0 + 11.795 * vovrl);
            self.sfr[6] = 0.0480 * vovrl / (1.0 + 0.0480 * vovrl);
            self.sfr[7] = 0.0242 * vovrl / (1.0 + 0.0242 * vovrl);
        } else {
            self.sfr[3] = 0.9999;
            self.sfr[4] = 0.999;
            self.sfr[5] = 0.999;
            self.sfr[6] = 0.99;
            self.sfr[7] = 0.98;
        }

        let mut fin = [0.0; 8];
        for i in 0..8 {
            fin[i] = self.fcm[3][i] + self.fcm[10][i];
        }
        self.ftm[4] = 0.0;
        self.ftm[11] = 0.0;
        for i in 0..8 {
            self.fcm[4][i] = self.sfr[i] * fin[i];
            self.fcm[11][i] = fin[i] - self.fcm[4][i];
            self.ftm[4] += self.fcm[4][i];
            self.ftm[11] += self.fcm[11][i];
        }
        for i in 0..8 {
            self.xst[4][i] = self.fcm[4][i] / self.ftm[4];
            self.xst[11][i] = self.fcm[11][i] / self.ftm[11];
        }
        self.tst[4] = self.tcc;
        self.tst[11] = self.tcc;
        self.hst[4] = self.tesub1(&self.xst[4], self.tst[4], 1);
        self.hst[11] = self.tesub1(&self.xst[11], self.tst[11], 0);

        self.ftm[6] = self.ftm[5];
        self.hst[6] = self.hst[5];
        self.tst[6] = self.tst[5];
        for i in 0..8 {
            self.xst[6][i] = self.xst[5][i];
            self.fcm[6][i] = self.fcm[5][i];
        }

        let uarlev = if self.vlr / 7.8 > 50.0 {
            1.0
        } else if self.vlr / 7.8 < 10.0 {
            0.0
        } else {
            0.025 * self.vlr / 7.8 - 0.25
        };
        self.uar = uarlev * (-0.5 * self.agsp * self.agsp + 2.75 * self.agsp - 2.5) * 855490.0e-6;
        self.qur = self.uar * (self.twr - self.tcr) * (1.0 - 0.35 * self.tesub8(9, time));
        let uas = 0.404655 * (1.0 - 1.0 / (1.0 + (self.ftm[7] / 3528.73).powi(4)));
        self.qus = uas * (self.tws - self.tst[7]) * (1.0 - 0.25 * self.tesub8(10, time));
        self.quc = 0.0;
        if self.tcc < 100.0 {
            self.quc = uac * (100.0 - self.tcc);
        }

        self.xmeas[0] = self.ftm[2] * 0.359 / 35.3145;
        self.xmeas[1] = self.ftm[0] * self.xmws[0] * 0.454;
        self.xmeas[2] = self.ftm[1] * self.xmws[1] * 0.454;
        self.xmeas[3] = self.ftm[3] * 0.359 / 35.3145;
        self.xmeas[4] = self.ftm[8] * 0.359 / 35.3145;
        self.xmeas[5] = self.ftm[5] * 0.359 / 35.3145;
        self.xmeas[6] = (self.ptr - 760.0) / 760.0 * 101.325;
        self.xmeas[7] = (self.vlr - 84.6) / 666.7 * 100.0;
        self.xmeas[8] = self.tcr;
        self.xmeas[9] = self.ftm[9] * 0.359 / 35.3145;
        self.xmeas[10] = self.tcs;
        self.xmeas[11] = (self.vls - 27.5) / 290.0 * 100.0;
        self.xmeas[12] = (self.pts - 760.0) / 760.0 * 101.325;
        self.xmeas[13] = self.ftm[10] / self.dls / 35.3145;
        self.xmeas[14] = (self.vlc - 78.25) / self.vtc * 100.0;
        self.xmeas[15] = (self.ptv - 760.0) / 760.0 * 101.325;
        self.xmeas[16] = self.ftm[12] / self.dlc / 35.3145;
        self.xmeas[17] = self.tcc;
        self.xmeas[18] = self.quc * 1.04e3 * 0.454;
        self.xmeas[19] = self.cpdh * 0.29307e3;
        self.xmeas[20] = self.twr;
        self.xmeas[21] = self.tws;

        self.isd = 0;
        if self.xmeas[6] > 3000.0 {
            self.isd = 1;
        }
        if self.vlr / 35.3145 > 24.0 {
            self.isd = 1;
        }
        if self.vlr / 35.3145 < 2.0 {
            self.isd = 1;
        }
        if self.xmeas[8] > 175.0 {
            self.isd = 1;
        }
        if self.vls / 35.3145 > 12.0 {
            self.isd = 1;
        }
        if self.vls / 35.3145 < 1.0 {
            self.isd = 1;
        }
        if self.vlc / 35.3145 > 8.0 {
            self.isd = 1;
        }
        if self.vlc / 35.3145 < 1.0 {
            self.isd = 1;
        }
        if self.time > 0.0 && self.isd == 0 {
            for i in 0..22 {
                let xmns = self.tesub6(self.xns[i]);
                self.xmeas[i] += xmns;
            }
        }

        let mut xcmp = [0.0; 41];
        xcmp[22] = self.xst[6][0] * 100.0;
        xcmp[23] = self.xst[6][1] * 100.0;
        xcmp[24] = self.xst[6][2] * 100.0;
        xcmp[25] = self.xst[6][3] * 100.0;
        xcmp[26] = self.xst[6][4] * 100.0;
        xcmp[27] = self.xst[6][5] * 100.0;
        xcmp[28] = self.xst[9][0] * 100.0;
        xcmp[29] = self.xst[9][1] * 100.0;
        xcmp[30] = self.xst[9][2] * 100.0;
        xcmp[31] = self.xst[9][3] * 100.0;
        xcmp[32] = self.xst[9][4] * 100.0;
        xcmp[33] = self.xst[9][5] * 100.0;
        xcmp[34] = self.xst[9][6] * 100.0;
        xcmp[35] = self.xst[9][7] * 100.0;
        xcmp[36] = self.xst[12][3] * 100.0;
        xcmp[37] = self.xst[12][4] * 100.0;
        xcmp[38] = self.xst[12][5] * 100.0;
        xcmp[39] = self.xst[12][6] * 100.0;
        xcmp[40] = self.xst[12][7] * 100.0;

        if self.time == 0.0 {
            for i in 22..41 {
                self.xdel[i] = xcmp[i];
                self.xmeas[i] = xcmp[i];
            }
            self.tgas = 0.1;
            self.tprod = 0.25;
        }
        if self.time >= self.tgas {
            for i in 22..36 {
                self.xmeas[i] = self.xdel[i];
                let xmns = self.tesub6(self.xns[i]);
                self.xmeas[i] += xmns;
                self.xdel[i] = xcmp[i];
            }
            self.tgas += 0.1;
        }
        if self.time >= self.tprod {
            for i in 36..41 {
                self.xmeas[i] = self.xdel[i];
                let xmns = self.tesub6(self.xns[i]);
                self.xmeas[i] += xmns;
                self.xdel[i] = xcmp[i];
            }
            self.tprod += 0.25;
        }

        for i in 0..8 {
            self.yp[i] = self.fcm[6][i] - self.fcm[7][i] + self.crxr[i];
            self.yp[i + 9] = self.fcm[7][i] - self.fcm[8][i] - self.fcm[9][i] - self.fcm[10][i];
            self.yp[i + 18] = self.fcm[11][i] - self.fcm[12][i];
            self.yp[i + 27] =
                self.fcm[0][i] + self.fcm[1][i] + self.fcm[2][i] + self.fcm[4][i] + self.fcm[8][i]
                    - self.fcm[5][i];
        }
        self.yp[8] = self.hst[6] * self.ftm[6] - self.hst[7] * self.ftm[7] + self.rh + self.qur;
        self.yp[17] = self.hst[7] * self.ftm[7]
            - self.hst[8] * self.ftm[8]
            - self.hst[9] * self.ftm[9]
            - self.hst[10] * self.ftm[10]
            + self.qus;
        self.yp[26] = self.hst[3] * self.ftm[3] + self.hst[10] * self.ftm[10]
            - self.hst[4] * self.ftm[4]
            - self.hst[12] * self.ftm[12]
            + self.quc;
        self.yp[35] = self.hst[0] * self.ftm[0]
            + self.hst[1] * self.ftm[1]
            + self.hst[2] * self.ftm[2]
            + self.hst[4] * self.ftm[4]
            + self.hst[8] * self.ftm[8]
            - self.hst[5] * self.ftm[5];
        self.yp[36] =
            (self.fwr * 500.53 * (self.tcwr - self.twr) - self.qur * 1.0e6 / 1.8) / self.hwr;
        self.yp[37] =
            (self.fws * 500.53 * (self.tcws - self.tws) - self.qus * 1.0e6 / 1.8) / self.hws;

        self.ivst[9] = self.idv[13];
        self.ivst[10] = self.idv[14];
        self.ivst[4] = self.idv[18];
        self.ivst[6] = self.idv[18];
        self.ivst[7] = self.idv[18];
        self.ivst[8] = self.idv[18];
        for i in 0..12 {
            if self.time == 0.0
                || (self.vcv[i] - self.xmv[i]).abs() > self.vst[i] * f64::from(self.ivst[i])
            {
                self.vcv[i] = self.xmv[i];
            }
            self.vcv[i] = self.vcv[i].clamp(0.0, 100.0);
            self.yp[i + 38] = (self.vcv[i] - vpos[i]) / self.vtau[i];
        }
        if self.isd != 0 {
            self.yp = [0.0; N_STATES];
        }
    }

    fn tesub1(&self, z: &[f64; 8], t: f64, ity: i32) -> f64 {
        let mut h = 0.0;
        if ity == 0 {
            for i in 0..8 {
                let mut hi = t * (self.ah[i] + self.bh[i] * t / 2.0 + self.ch[i] * t.powi(2) / 3.0);
                hi *= 1.8;
                h += z[i] * self.xmw[i] * hi;
            }
        } else {
            for i in 0..8 {
                let mut hi = t * (self.ag[i] + self.bg[i] * t / 2.0 + self.cg[i] * t.powi(2) / 3.0);
                hi *= 1.8;
                hi += self.av[i];
                h += z[i] * self.xmw[i] * hi;
            }
        }
        if ity == 2 {
            let r = 3.57696e-6;
            h -= r * (t + 273.15);
        }
        h
    }

    fn tesub2(&self, z: &[f64; 8], t: &mut f64, h: f64, ity: i32) {
        let tin = *t;
        for _ in 0..100 {
            let htest = self.tesub1(z, *t, ity);
            let err = htest - h;
            let dh = self.tesub3(z, *t, ity);
            let dt = -err / dh;
            *t += dt;
            if dt.abs() < 1.0e-12 {
                return;
            }
        }
        *t = tin;
    }

    fn tesub3(&self, z: &[f64; 8], t: f64, ity: i32) -> f64 {
        let mut dh = 0.0;
        if ity == 0 {
            for i in 0..8 {
                let mut dhi = self.ah[i] + self.bh[i] * t + self.ch[i] * t.powi(2);
                dhi *= 1.8;
                dh += z[i] * self.xmw[i] * dhi;
            }
        } else {
            for i in 0..8 {
                let mut dhi = self.ag[i] + self.bg[i] * t + self.cg[i] * t.powi(2);
                dhi *= 1.8;
                dh += z[i] * self.xmw[i] * dhi;
            }
        }
        if ity == 2 {
            dh -= 3.57696e-6;
        }
        dh
    }

    fn tesub4(&self, x: &[f64; 8], t: f64) -> f64 {
        let mut v = 0.0;
        for i in 0..8 {
            v += x[i] * self.xmw[i] / (self.ad[i] + (self.bd[i] + self.cd[i] * t) * t);
        }
        1.0 / v
    }

    fn tesub5(
        &mut self,
        s: f64,
        sp: f64,
        tlast: f64,
        hspan: f64,
        hzero: f64,
        sspan: f64,
        szero: f64,
        spspan: f64,
        idvflag: i32,
    ) -> (f64, f64, f64, f64, f64) {
        let i = -1;
        let h = hspan * self.tesub7(i) + hzero;
        let s1 = sspan * self.tesub7(i) * f64::from(idvflag) + szero;
        let s1p = spspan * self.tesub7(i) * f64::from(idvflag);
        let adist = s;
        let bdist = sp;
        let cdist = (3.0 * (s1 - s) - h * (s1p + 2.0 * sp)) / (h * h);
        let ddist = (2.0 * (s - s1) + h * (s1p + sp)) / (h * h * h);
        let tnext = tlast + h;
        (adist, bdist, cdist, ddist, tnext)
    }

    fn tesub6(&mut self, std: f64) -> f64 {
        let mut x = 0.0;
        for i in 1..=12 {
            x += self.tesub7(i);
        }
        (x - 6.0) * std
    }

    fn tesub7(&mut self, i: i32) -> f64 {
        const MODULUS: f64 = 4_294_967_296.0;
        self.g = (self.g * 9_228_907.0) % MODULUS;
        if i >= 0 {
            self.g / MODULUS
        } else {
            2.0 * self.g / MODULUS - 1.0
        }
    }

    fn tesub8(&self, i: usize, t: f64) -> f64 {
        let h = t - self.tlast[i];
        self.adist[i] + h * (self.bdist[i] + h * (self.cdist[i] + h * self.ddist[i]))
    }
}

impl TennesseeEastmanProcess {
    pub fn xmeas_names() -> [&'static str; N_XMEAS] {
        [
            "A Feed (stream 1)",
            "D Feed (stream 2)",
            "E Feed (stream 3)",
            "A and C Feed (stream 4)",
            "Recycle Flow (stream 8)",
            "Reactor Feed Rate (stream 6)",
            "Reactor Pressure",
            "Reactor Level",
            "Reactor Temperature",
            "Purge Rate (stream 9)",
            "Product Sep Temp",
            "Product Sep Level",
            "Prod Sep Pressure",
            "Prod Sep Underflow (stream 10)",
            "Stripper Level",
            "Stripper Pressure",
            "Stripper Underflow (stream 11)",
            "Stripper Temperature",
            "Stripper Steam Flow",
            "Compressor Work",
            "Reactor Cooling Water Outlet Temp",
            "Separator Cooling Water Outlet Temp",
            "Component A (stream 6)",
            "Component B (stream 6)",
            "Component C (stream 6)",
            "Component D (stream 6)",
            "Component E (stream 6)",
            "Component F (stream 6)",
            "Component A (stream 9)",
            "Component B (stream 9)",
            "Component C (stream 9)",
            "Component D (stream 9)",
            "Component E (stream 9)",
            "Component F (stream 9)",
            "Component G (stream 9)",
            "Component H (stream 9)",
            "Component D (stream 11)",
            "Component E (stream 11)",
            "Component F (stream 11)",
            "Component G (stream 11)",
            "Component H (stream 11)",
        ]
    }

    pub fn xmv_names() -> [&'static str; N_XMV] {
        [
            "D Feed Flow (stream 2)",
            "E Feed Flow (stream 3)",
            "A Feed Flow (stream 1)",
            "A and C Feed Flow (stream 4)",
            "Compressor Recycle Valve",
            "Purge Valve (stream 9)",
            "Separator Pot Liquid Flow (stream 10)",
            "Stripper Liquid Product Flow (stream 11)",
            "Stripper Steam Valve",
            "Reactor Cooling Water Flow",
            "Condenser Cooling Water Flow",
            "Agitator Speed",
        ]
    }

    pub fn idv_names() -> [&'static str; N_IDV] {
        [
            "A/C Feed Ratio, B Composition Constant (Stream 4)",
            "B Composition, A/C Ratio Constant (Stream 4)",
            "D Feed Temperature (Stream 2)",
            "Reactor Cooling Water Inlet Temperature",
            "Condenser Cooling Water Inlet Temperature",
            "A Feed Loss (Stream 1)",
            "C Header Pressure Loss (Stream 4)",
            "A, B, C Feed Composition (Stream 4)",
            "D Feed Temperature (Stream 2)",
            "C Feed Temperature (Stream 4)",
            "Reactor Cooling Water Inlet Temperature",
            "Condenser Cooling Water Inlet Temperature",
            "Reaction Kinetics",
            "Reactor Cooling Water Valve",
            "Condenser Cooling Water Valve",
            "Unknown",
            "Unknown",
            "Unknown",
            "Unknown",
            "Unknown",
        ]
    }
}

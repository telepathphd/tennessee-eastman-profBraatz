use tennessee_eastman::{
    default_delta_t, ClosedLoopConfig, PlantWideController, TennesseeEastmanProcess,
    DEFAULT_RNG_SEED, N_STATES,
};

const YY0: [f64; N_STATES] = [
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

#[test]
fn default_delta_t_is_one_second_in_hours() {
    let dt = default_delta_t();
    assert!((dt - 1.0 / 3600.0).abs() < 1e-18);
    assert!((dt * 3600.0 - 1.0).abs() < 1e-12);
}

#[test]
fn teinit_keeps_written_initial_states() {
    let mut p = TennesseeEastmanProcess::new();
    p.teinit();
    for i in 0..N_STATES {
        let err = (p.yy[i] - YY0[i]).abs();
        assert!(
            err < 1e-14,
            "YY[{}] drifted from the written IC: got={} written={}",
            i + 1,
            p.yy[i],
            YY0[i]
        );
    }
}

#[test]
fn teinit_base_case_is_near_published_operating_point() {
    let mut p = TennesseeEastmanProcess::new();
    p.teinit();
    let x = p.xmeas();
    assert!(!p.is_shutdown());
    // Downs & Vogel / Braatz base case (closed-loop setpoints in temain_mod.f).
    assert!((x[0] - 0.25052).abs() < 0.02, "A feed {}", x[0]);
    assert!((x[1] - 3664.0).abs() < 50.0, "D feed {}", x[1]);
    assert!((x[2] - 4509.3).abs() < 80.0, "E feed {}", x[2]);
    assert!((x[6] - 2705.0).abs() < 20.0, "reactor pressure {}", x[6]);
    assert!((x[7] - 75.0).abs() < 2.0, "reactor level {}", x[7]);
    assert!((x[8] - 120.4).abs() < 0.5, "reactor temperature {}", x[8]);
    assert!((x[11] - 50.0).abs() < 2.0, "separator level {}", x[11]);
    assert!((x[14] - 50.0).abs() < 2.0, "stripper level {}", x[14]);
}

#[test]
fn one_euler_step_advances_time_without_noise_at_t0() {
    let mut p = TennesseeEastmanProcess::new();
    p.teinit();
    assert_eq!(p.g, DEFAULT_RNG_SEED);
    p.integrate(default_delta_t());
    assert_eq!(
        p.g, DEFAULT_RNG_SEED,
        "INTGTR at TIME=0 must not draw measurement noise"
    );
    assert!((p.time - default_delta_t()).abs() < 1e-18);
    assert!(!p.is_shutdown());
    assert!(
        (p.xmeas()[8] - 120.4).abs() < 1.0,
        "reactor temperature after 1 s: {}",
        p.xmeas()[8]
    );

    p.tefunc();
    assert_ne!(p.g, DEFAULT_RNG_SEED, "TESUB7 must advance after t>0");
}

#[test]
fn observation_layout_is_41_meas_plus_11_mv() {
    let mut p = TennesseeEastmanProcess::new();
    p.teinit();
    let obs = p.observation();
    assert_eq!(obs.len(), 52);
    assert_eq!(obs[0], p.xmeas()[0]);
    assert_eq!(obs[40], p.xmeas()[40]);
    assert_eq!(obs[41], p.xmv()[0]);
    assert_eq!(obs[51], p.xmv()[10]);
}

#[test]
fn closed_loop_short_run_stays_near_base_case() {
    let mut p = TennesseeEastmanProcess::new();
    p.teinit();
    let cfg = ClosedLoopConfig {
        npts: 360,
        sspts: 10_000,
        delta_t: default_delta_t(),
        idv_after_ss: vec![],
    };
    let mut ctrl = PlantWideController::new(cfg.delta_t);
    PlantWideController::apply_base_xmv(&mut p);
    for i in 1..=cfg.npts {
        ctrl.step(&mut p, i);
        p.integrate(cfg.delta_t);
        ctrl.constrain_hand(&mut p);
        assert!(!p.is_shutdown(), "shutdown at step {i}");
    }
    let t = p.xmeas()[8];
    assert!(
        (t - 120.4).abs() < 5.0,
        "reactor temperature drifted too far: {t}"
    );
}

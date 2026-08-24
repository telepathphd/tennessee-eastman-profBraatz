//! Run an identification experiment and write mimo-sim CSV + meta.json.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tennessee_eastman::experiment::{
    default_reactor_temp_mapping, run, CvChannel, ExperimentRequest, LoopMode, MvChannel,
    SchedulePoint,
};
use tennessee_eastman::simulate::{Injection, SimMode, SimulationRequest};

fn main() {
    let opts = Opts::parse();
    let (mv, cv) = if opts.mv_channels.is_empty() {
        default_reactor_temp_mapping()
    } else {
        (opts.mv_channels, opts.cv_channels)
    };

    let mut setpoint_schedule = opts.setpoint_schedule;
    if setpoint_schedule.is_empty() && opts.gbn_amplitude > 0.0 {
        eprintln!(
            "warning: --gbn-amplitude set but no schedule; use te_client Python to build schedules"
        );
    }

    let req = ExperimentRequest {
        sim: SimulationRequest {
            mode: SimMode::ClosedLoop,
            npts: opts.npts,
            record_every: opts.record_every,
            seed: opts.seed,
            setpoints: opts.setpoints,
            held_setpoints: opts.held,
            injections: opts.injections,
            open_loop_xmv: BTreeMap::new(),
            open_loop_stripper_sp: None,
        },
        setpoint_schedule,
        xmv_schedule: vec![],
        loop_mode: opts.loop_mode,
        mv_channels: mv,
        cv_channels: cv,
        export_dir: Some(opts.output.clone()),
        export_stem: opts.stem.clone(),
        full_record: true,
    };

    match run(&req) {
        Ok(out) => {
            eprintln!("steps_run={}", out.sim.steps_run);
            if let Some(p) = &out.csv_path {
                eprintln!("wrote {}", p.display());
            }
            if let Some(p) = &out.meta_path {
                eprintln!("wrote {}", p.display());
            }
            if out.sim.shutdown {
                eprintln!("shutdown @ {} s", out.sim.shutdown_time_s.unwrap_or(0));
            }
        }
        Err(err) => {
            eprintln!("error: {}", err.0);
            std::process::exit(1);
        }
    }
}

struct Opts {
    npts: usize,
    record_every: usize,
    seed: f64,
    output: PathBuf,
    stem: String,
    setpoints: BTreeMap<usize, f64>,
    held: Vec<usize>,
    injections: Vec<Injection>,
    setpoint_schedule: Vec<SchedulePoint>,
    loop_mode: BTreeMap<usize, LoopMode>,
    gbn_amplitude: f64,
    mv_channels: Vec<MvChannel>,
    cv_channels: Vec<CvChannel>,
}

impl Opts {
    fn parse() -> Self {
        let mut npts = 7200;
        let mut record_every = 60;
        let mut seed = tennessee_eastman::DEFAULT_RNG_SEED;
        let mut output = PathBuf::from(".");
        let mut stem = "te_experiment".to_string();
        let mut setpoints = BTreeMap::new();
        let mut held = vec![18usize];
        let mut injections = Vec::new();
        let mut setpoint_schedule = Vec::new();
        let mut loop_mode = BTreeMap::new();
        let mut gbn_amplitude = 0.0;
        let mut mv_channels = Vec::new();
        let mut cv_channels = Vec::new();

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--npts" => npts = args.next().expect("--npts").parse().expect("npts"),
                "--record-every" => {
                    record_every = args.next().expect("--record-every").parse().expect("every")
                }
                "--seed" => seed = args.next().expect("--seed").parse().expect("seed"),
                "--output" => output = PathBuf::from(args.next().expect("--output")),
                "--stem" => stem = args.next().expect("--stem"),
                "--held" => {
                    held = args
                        .next()
                        .expect("--held")
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect()
                }
                "--step" => {
                    let spec = args.next().expect("--step");
                    let (step, rest) = spec.split_once(':').expect("step format STEP:SETPT:VALUE");
                    let (setpt, val) = rest.split_once(':').expect("step format STEP:SETPT:VALUE");
                    setpoint_schedule.push(SchedulePoint {
                        start_step: step.parse().expect("step"),
                        n: setpt.parse().expect("setpt"),
                        value: val.parse().expect("value"),
                    });
                }
                "--idv" => {
                    let spec = args.next().expect("--idv");
                    let (idv, step) = spec.split_once('@').map(|(a, b)| (a, b)).unwrap_or((spec.as_str(), "1"));
                    injections.push(Injection {
                        idv: idv.parse().expect("idv"),
                        start_step: step.parse().unwrap_or(1),
                    });
                }
                "--gbn-amplitude" => {
                    gbn_amplitude = args.next().expect("--gbn-amplitude").parse().expect("amp")
                }
                "-h" | "--help" => {
                    eprintln!(
                        "te-experiment [--npts 7200] [--record-every 60] [--output DIR] [--stem NAME]\n\
                         [--held 18] [--step STEP:SETPT:VALUE] [--idv N[@STEP]]\n\
                         Prefer python/te_client for GBN schedules."
                    );
                    std::process::exit(0);
                }
                other => panic!("unknown argument: {other}"),
            }
        }

        Self {
            npts,
            record_every,
            seed,
            output,
            stem,
            setpoints,
            held,
            injections,
            setpoint_schedule,
            loop_mode,
            gbn_amplitude,
            mv_channels,
            cv_channels,
        }
    }
}

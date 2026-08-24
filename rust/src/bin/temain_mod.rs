//! Closed-loop demonstration (`archive/temain_mod.f`).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use tennessee_eastman::{
    ClosedLoopConfig, PlantWideController, TennesseeEastmanProcess, DEFAULT_RNG_SEED,
};

fn main() {
    let opts = Opts::parse();
    let mut process = TennesseeEastmanProcess::with_seed(opts.seed);
    process.teinit();

    let mut ctrl = PlantWideController::new(opts.cfg.delta_t);
    PlantWideController::apply_base_xmv(&mut process);
    for i in 0..20 {
        process.set_idv(i + 1, false);
    }

    std::fs::create_dir_all(&opts.output).expect("create output directory");
    let mut files = OutputFiles::open(&opts.output, opts.overwrite);

    for i in 1..=opts.cfg.npts {
        if i >= opts.cfg.sspts {
            for &k in &opts.cfg.idv_after_ss {
                if k > 0 {
                    process.set_idv(k, true);
                }
            }
        }
        ctrl.step(&mut process, i);
        if i % 5000 == 0 {
            println!("Simulation time (in seconds) = {i}");
        }
        if i % 180 == 0 {
            files.write(i, &process);
        }
        process.integrate(opts.cfg.delta_t);
        ctrl.constrain_hand(&mut process);
        if process.is_shutdown() {
            eprintln!("process shutdown at t = {} h", process.time);
            break;
        }
    }
    println!("Simulation is done. ");
}

struct Opts {
    cfg: ClosedLoopConfig,
    seed: f64,
    output: PathBuf,
    overwrite: bool,
}

impl Opts {
    fn parse() -> Self {
        let mut cfg = ClosedLoopConfig::default();
        let mut seed = DEFAULT_RNG_SEED;
        let mut output = PathBuf::from(".");
        let mut overwrite = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--npts" => cfg.npts = next_parse(&mut args, "--npts"),
                "--sspts" => cfg.sspts = next_parse(&mut args, "--sspts"),
                "--seed" => seed = next_parse(&mut args, "--seed"),
                "--output" => {
                    output = PathBuf::from(args.next().expect("--output needs a path"));
                }
                "--idv" => {
                    let spec = args.next().expect("--idv needs a value");
                    cfg.idv_after_ss = if spec == "0" {
                        Vec::new()
                    } else {
                        spec.split(',')
                            .map(|s| s.parse().expect("idv must be integers"))
                            .collect()
                    };
                }
                "--overwrite" => overwrite = true,
                "-h" | "--help" => {
                    eprintln!(
                        "temain_mod [--npts N] [--sspts N] [--idv K[,K...]] [--seed G]\n\
                         [--output DIR] [--overwrite]\n\
                         defaults: npts=172800 sspts=28800 idv=12 seed={DEFAULT_RNG_SEED}"
                    );
                    std::process::exit(0);
                }
                other => panic!("unknown argument: {other}"),
            }
        }
        Self {
            cfg,
            seed,
            output,
            overwrite,
        }
    }
}

fn next_parse<T: std::str::FromStr>(args: &mut impl Iterator<Item = String>, flag: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    args.next()
        .unwrap_or_else(|| panic!("{flag} needs a value"))
        .parse()
        .unwrap_or_else(|_| panic!("{flag} has an invalid value"))
}

struct OutputFiles {
    inc: BufWriter<File>,
    mv: [BufWriter<File>; 3],
    me: [BufWriter<File>; 11],
}

impl OutputFiles {
    fn open(dir: &Path, overwrite: bool) -> Self {
        let open = |name: &str| {
            let path = dir.join(name);
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true);
            if overwrite {
                opts.create(true).truncate(true);
            } else {
                opts.create_new(true);
            }
            BufWriter::new(opts.open(&path).unwrap_or_else(|e| {
                panic!(
                    "cannot open {}: {e} (use --overwrite to replace)",
                    path.display()
                )
            }))
        };
        Self {
            inc: open("TE_data_inc.dat"),
            mv: [
                open("TE_data_mv1.dat"),
                open("TE_data_mv2.dat"),
                open("TE_data_mv3.dat"),
            ],
            me: [
                open("TE_data_me01.dat"),
                open("TE_data_me02.dat"),
                open("TE_data_me03.dat"),
                open("TE_data_me04.dat"),
                open("TE_data_me05.dat"),
                open("TE_data_me06.dat"),
                open("TE_data_me07.dat"),
                open("TE_data_me08.dat"),
                open("TE_data_me09.dat"),
                open("TE_data_me10.dat"),
                open("TE_data_me11.dat"),
            ],
        }
    }

    fn write(&mut self, i: usize, process: &TennesseeEastmanProcess) {
        writeln!(self.inc, " {i:6}").unwrap();
        let mv = process.xmv();
        let x = process.xmeas();
        write4(&mut self.mv[0], &mv[0..4]);
        write4(&mut self.mv[1], &mv[4..8]);
        write4(&mut self.mv[2], &mv[8..12]);
        write4(&mut self.me[0], &x[0..4]);
        write4(&mut self.me[1], &x[4..8]);
        write4(&mut self.me[2], &x[8..12]);
        write4(&mut self.me[3], &x[12..16]);
        write4(&mut self.me[4], &x[16..20]);
        write4(&mut self.me[5], &x[20..24]);
        write4(&mut self.me[6], &x[24..28]);
        write4(&mut self.me[7], &x[28..32]);
        write4(&mut self.me[8], &x[32..36]);
        write4(&mut self.me[9], &x[36..40]);
        writeln!(self.me[10], " {:13.5E}", x[40]).unwrap();
    }
}

fn write4(w: &mut BufWriter<File>, v: &[f64]) {
    writeln!(
        w,
        " {:13.5E}  {:13.5E}  {:13.5E}  {:13.5E}",
        v[0], v[1], v[2], v[3]
    )
    .unwrap();
}

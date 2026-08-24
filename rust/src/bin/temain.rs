//! Open-loop demonstration (`temain.f`).

use tennessee_eastman::{default_delta_t, StripperLevelController, TennesseeEastmanProcess};

fn main() {
    let npts = parse_npts();
    let dt = default_delta_t();
    let mut process = TennesseeEastmanProcess::new();
    process.teinit();

    let mut ctrl = StripperLevelController::from_process(&process);
    process.set_xmv(10, 38.0);
    for i in 0..20 {
        process.set_idv(i + 1, false);
    }

    for _ in 0..npts {
        ctrl.apply(&mut process, dt);
        let x = process.xmeas();
        let mv = process.xmv();
        println!(
            "Reac Temp = {:6.2}  Stripper Lev = {:6.2}  Sripper Underflow = {:6.2}",
            x[8], x[14], mv[7]
        );
        process.integrate(dt);
        if process.is_shutdown() {
            eprintln!("process shutdown");
            break;
        }
    }
}

fn parse_npts() -> usize {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--npts" => {
                return args
                    .next()
                    .expect("--npts needs a value")
                    .parse()
                    .expect("npts must be an integer");
            }
            "-h" | "--help" => {
                eprintln!("temain [--npts N]   (default N=1000, 1 s/step)");
                std::process::exit(0);
            }
            other => panic!("unknown argument: {other}"),
        }
    }
    1000
}

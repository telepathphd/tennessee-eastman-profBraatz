# Tennessee Eastman Process

Rust replica of the Downs & Vogel / Russell–Chiang–Braatz plant-wide process-control test problem, with a local Vue operator console.

The original Illinois Fortran 77 sources and FDD datasets are kept as historical reference in [`archive/`](archive/).

## Layout

Directory | Role
--------- | ----
[`rust/`](rust/) | Process replica, open/closed-loop CLI, local console backend
[`web/`](web/) | Vue operator desk (P&ID, strip chart, setpoints, scheduled injections)
[`archive/`](archive/) | Original `teprob.f` / `temain.f` / `temain_mod.f` and `d*.dat` files

COMMON blocks become `TennesseeEastmanProcess`. `TEINIT` / `TEFUNC` / `TESUBi` keep the original equations in IEEE-754 double (constants as written, not Fortran default-kind `REAL` rounding). The `TESUB7` LCG is kept so measurement noise and random-variation disturbances stay reproducible. Trajectories are not bit-identical to gfortran dumps.

## Console (`te-console` + `web/`)

Vue operator desk on the native replica: setpoints, scheduled disturbance injections, run, P&ID + strip chart. Offline after `npm run build`.

```bash
cd web && npm install && npm run build
cd ../rust && cargo run --release --bin te-console
# http://127.0.0.1:8787
```

During UI work, keep the API on 8787 and `cd web && npm run dev` (Vite proxies `/api`).

## CLI

```bash
cd rust
cargo test --release
cargo run --release --bin temain -- --npts 1000
cargo run --release --bin temain_mod -- --npts 172800 --sspts 28800 --idv 12 --output . --overwrite
```

`temain_mod` flags map to the Fortran edit points: `--npts` (`NPTS`), `--sspts` (`SSPTS`), `--idv` (disturbances after steady state; `0` disables them), `--seed` (`G` in `teprob.f`). Output files use the same `TE_data_*.dat` names as the original closed-loop program, written to `--output` instead of `~/`.

## Library

```rust
use tennessee_eastman::{default_delta_t, TennesseeEastmanProcess};

let mut plant = TennesseeEastmanProcess::new();
plant.teinit();
plant.set_idv(12, true);
plant.integrate(default_delta_t());
let x = plant.observation(); // XMEAS(1..41), XMV(1..11)
```

## Manipulated Variables

Variable | Description
-------- | -----------
`XMV(1)`  | D Feed Flow (stream 2)            (Corrected Order)
`XMV(2)`  | E Feed Flow (stream 3)            (Corrected Order)
`XMV(3)`  | A Feed Flow (stream 1)            (Corrected Order)
`XMV(4)`  | A and C Feed Flow (stream 4)
`XMV(5)`  | Compressor Recycle Valve
`XMV(6)`  | Purge Valve (stream 9)
`XMV(7)`  | Separator Pot Liquid Flow (stream 10)
`XMV(8)`  | Stripper Liquid Product Flow (stream 11)
`XMV(9)`  | Stripper Steam Valve
`XMV(10)` | Reactor Cooling Water Flow
`XMV(11)` | Condenser Cooling Water Flow
`XMV(12)` | Agitator Speed

## Continuous Process Measurements

Variable | Description | unit
-------- | ----------- | ----
`XMEAS(1)`  | A Feed  (stream 1)                  | kscmh
`XMEAS(2)`  | D Feed  (stream 2)                  | kg/hr
`XMEAS(3)`  | E Feed  (stream 3)                  | kg/hr
`XMEAS(4)`  | A and C Feed  (stream 4)            | kscmh
`XMEAS(5)`  | Recycle Flow  (stream 8)            | kscmh
`XMEAS(6)`  | Reactor Feed Rate  (stream 6)       | kscmh
`XMEAS(7)`  | Reactor Pressure                    | kPa gauge
`XMEAS(8)`  | Reactor Level                       | %
`XMEAS(9)`  | Reactor Temperature                 | Deg C
`XMEAS(10)` | Purge Rate (stream 9)               | kscmh
`XMEAS(11)` | Product Sep Temp                    | Deg C
`XMEAS(12)` | Product Sep Level                   | %
`XMEAS(13)` | Prod Sep Pressure                   | kPa gauge
`XMEAS(14)` | Prod Sep Underflow (stream 10)      | m3/hr
`XMEAS(15)` | Stripper Level                      | %
`XMEAS(16)` | Stripper Pressure                   | kPa gauge
`XMEAS(17)` | Stripper Underflow (stream 11)      | m3/hr
`XMEAS(18)` | Stripper Temperature                | Deg C
`XMEAS(19)` | Stripper Steam Flow                 | kg/hr
`XMEAS(20)` | Compressor Work                     | kW
`XMEAS(21)` | Reactor Cooling Water Outlet Temp   | Deg C
`XMEAS(22)` | Separator Cooling Water Outlet Temp | Deg C

## Sampled Process Measurements

- Reactor Feed Analysis (Stream 6)
  > - Sampling Frequency = 0.1 hr
  > - Dead Time = 0.1 hr
  > - Mole %

	Variable | Description
	-------- | -----------
	`XMEAS(23)` | Component A
	`XMEAS(24)` | Component B
	`XMEAS(25)` | Component C
	`XMEAS(26)` | Component D
	`XMEAS(27)` | Component E
	`XMEAS(28)` | Component F

- Purge Gas Analysis (Stream 9)
  > - Sampling Frequency = 0.1 hr
  > - Dead Time = 0.1 hr
  > - Mole %

	Variable | Description
	-------- | -----------
	`XMEAS(29)` | Component A
	`XMEAS(30)` | Component B
	`XMEAS(31)` | Component C
	`XMEAS(32)` | Component D
	`XMEAS(33)` | Component E
	`XMEAS(34)` | Component F
	`XMEAS(35)` | Component G
	`XMEAS(36)` | Component H

- Product Analysis (Stream 11)
  > - Sampling Frequency = 0.25 hr
  > - Dead Time = 0.25 hr
  > - Mole %

	Variable | Description
	-------- | -----------
	`XMEAS(37)` | Component D
	`XMEAS(38)` | Component E
	`XMEAS(39)` | Component F
	`XMEAS(40)` | Component G
	`XMEAS(41)` | Component H

## Process Disturbances

Variable | Description
-------- | -----------
`IDV(1)`  | A/C Feed Ratio, B Composition Constant (Stream 4)          Step
`IDV(2)`  | B Composition, A/C Ratio Constant (Stream 4)               Step
`IDV(3)`  | D Feed Temperature (Stream 2)                              Step
`IDV(4)`  | Reactor Cooling Water Inlet Temperature                    Step
`IDV(5)`  | Condenser Cooling Water Inlet Temperature                  Step
`IDV(6)`  | A Feed Loss (Stream 1)                                     Step
`IDV(7)`  | C Header Pressure Loss - Reduced Availability (Stream 4)   Step
`IDV(8)`  | A, B, C Feed Composition (Stream 4)            Random Variation
`IDV(9)`  | D Feed Temperature (Stream 2)                  Random Variation
`IDV(10)` | C Feed Temperature (Stream 4)                  Random Variation
`IDV(11)` | Reactor Cooling Water Inlet Temperature        Random Variation
`IDV(12)` | Condenser Cooling Water Inlet Temperature      Random Variation
`IDV(13)` | Reaction Kinetics                                    Slow Drift
`IDV(14)` | Reactor Cooling Water Valve                            Sticking
`IDV(15)` | Condenser Cooling Water Valve                          Sticking
`IDV(16)` | Unknown | Stripper steam random variation (`TESUB8` → UAC)
`IDV(17)` | Unknown | Reactor heat removal random variation (`TESUB8` → QUR)
`IDV(18)` | Unknown | Separator heat removal random variation (`TESUB8` → QUS)
`IDV(19)` | Unknown | Sticking on XMV(5,7,8,9)
`IDV(20)` | Unknown | Reactor outlet flow random variation (`TESUB8` channel 12)

The published names stay **Unknown** for fault-detection benchmarks; the mechanisms above are how `TEFUNC` wires them. `d21.dat` is a dataset file, not a twenty-first `IDV` flag.

## License

Copyright (c) 1998-2002 The Board of Trustees of the University of Illinois. See [`LICENSE`](LICENSE).

Cite the original process and the closed-loop scheme:

- [J.J. Downs and E.F. Vogel, *A plant-wide industrial process control problem*, Computers and Chemical Engineering, 17:245-255 (1993)](https://doi.org/10.1016/0098-1354(93)80018-I)
- [E.L. Russell, L.H. Chiang, and R.D. Braatz. Data-driven Techniques for Fault Detection and Diagnosis in Chemical Processes, Springer-Verlag, London, 2000](https://doi.org/10.1007/978-1-4471-0409-4)
- [L.H. Chiang, E.L. Russell, and R.D. Braatz. Fault Detection and Diagnosis in Industrial Systems, Springer-Verlag, London, 2001](https://doi.org/10.1007/978-1-4471-0347-9)

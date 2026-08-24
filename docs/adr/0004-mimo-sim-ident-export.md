# mimo-sim CSV for identification export

Identification experiments export a single machine-readable time series aligned with [mimo-sim](https://github.com/telepathphd/SysID_Demo/tree/main/mimo-sim): columns `time, MV1…MVnu, CV1…CVny`, uniform sampling in engineering units, UTF-8 BOM optional. `procest(y, u, ts)` in that repo expects `u:(N,nu)`, `y:(N,ny)`, `ts` in seconds.

TE keeps its operator vocabulary in `CONTEXT.md` (Measurement, Manipulated Variable, Setpoint). `MV*`/`CV*` appear only in export files and Python glue. A channel map in `meta.json` records which `SETPT(n)` or `XMV(n)` became each `MV*` and which `XMEAS(n)` became each `CV*`.

Closed-loop identification excites **Setpoints** (or **Manipulated Variables** on manual loops); Braatz regulatory control stays in the loop. The exported `MV*` column is the applied excitation (Setpoint or valve command), not the inner cascade target unless mapped.

Excitation sequences (step, hold-time GBN) are generated in Python using mimo-sim semantics; this repo replays piecewise schedules and does not reimplement PRBS.

Full trajectories and experiment metadata (`meta.json`: seed, injections, loop Auto/Manual, analyzer delays) live beside the CSV. `TE_data_*.dat` and FDD `d*.dat` are not used for APC identification.

APC (phase 2) maps [python_hmpc](https://github.com/telepathphd/HMPC_Python) `HMPC.move` outputs to TE **Setpoints** at control period `Ts`, not directly to `XMV`, with Braatz inner loops still active.

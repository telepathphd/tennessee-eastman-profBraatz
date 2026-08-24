# Tennessee Eastman Process

Plant-wide process-control test problem (Downs & Vogel; Russell / Chiang / Braatz closed-loop). This glossary is the vocabulary for the replica, the local console, and operator-facing copy.

## Language

**Process**:
The Tennessee Eastman flowsheet: reactor, condenser, vapor-liquid separator, recycle compressor, and product stripper, with streams 1–11.
_Avoid_: plant (ambiguous with the company), model (the equations, not the unit)

**Measurement**:
A plant output in `XMEAS(1..41)`: continuous sensors plus the three composition analyzers.
_Avoid_: signal, tag (UI shorthand only), observation (the concatenated training vector)

**Manipulated Variable**:
A valve or agitator command in `XMV(1..12)`, 0–100%.
_Avoid_: input, actuator, MV (spell it out in prose)

**Disturbance**:
One of the twenty `IDV(n)` flags in the process: step, random variation, slow drift, sticking, or unknown.
_Avoid_: fault, accident, noise (measurement noise is separate)
_Note_: `IDV(16..20)` are published as unknown for FDD; `TEFUNC` still wires hidden mechanisms (stripper steam, heat removal, valve sticking, reactor outlet flow).

**Injection**:
Turning a Disturbance on at a chosen simulation step. The operator action; the flag is still a Disturbance.
_Avoid_: accident, attack, scenario (a bundle of injections plus setpoints)

**Setpoint**:
A `SETPT(n)` target in the plant-wide controller. Cascade outer loops write inner setpoints rather than valves.
_Avoid_: target, reference, SP (spell it out in prose)

**Shutdown**:
The interlock trip (`ISD ≠ 0`) that freezes derivatives when pressure, temperature, or liquid inventories leave the allowed band.
_Avoid_: crash, failure, stop

**Closed-loop**:
The Russell / Chiang / Braatz plant-wide controller (`temain_mod`) moving the Manipulated Variables.
_Avoid_: automatic, controlled

**Open-loop**:
The Downs & Vogel demonstration (`temain`) with only the stripper-level PI active.
_Avoid_: manual (valves can still be fixed by the operator)

**Steady-state interval**:
The steps before Injections start, matching Fortran `SSPTS`.
_Avoid_: warmup, burn-in

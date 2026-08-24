# te_hmpc_bridge

Connect **python_hmpc** `HMPC.move` / `get_result()` to the TE plant session API. APC manipulates **Setpoints** (e.g. `TIC1009`); Braatz inner loops remain active.

## Layout

- `HMPC MV` → TE `SETPT(n)` (held against cascade overwrite)
- `HMPC CV` ← TE `XMEAS(n)`

Requires `te-console` with `/api/session` and a discrete `HSS` model at control period `Ts = integrate_steps` seconds.

See `examples/closed_loop_demo.py` for a synthetic 1×1 plant smoke test (HMPC model ≠ TE physics).

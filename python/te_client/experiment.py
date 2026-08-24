"""Compose TE experiment requests with mimo-sim channel mapping."""

from __future__ import annotations

from typing import Any

from .signals import build_gbn_signal, linspace, schedule_from_signal


def default_reactor_mapping() -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Excite reactor temperature setpoint ``TIC1009`` / measure ``XMEAS(9)``."""
    mv = [{"kind": "setpoint", "n": 18, "tag": "TIC1009"}]
    cv = [{"n": 9, "tag": "TI1009"}]
    return mv, cv


def build_gbn_setpoint_experiment(
    *,
    npts: int = 7200,
    record_every: int = 60,
    seed: float = 0.0,
    setpt_n: int = 18,
    base: float = 120.4,
    amplitude: float = 2.0,
    t_min: float = 120.0,
    t_max: float = 600.0,
    held_setpoints: list[int] | None = None,
    mv_channels: list[dict[str, Any]] | None = None,
    cv_channels: list[dict[str, Any]] | None = None,
    export_dir: str | None = None,
    export_stem: str = "te_gbn_reactor",
) -> dict[str, Any]:
    """Build a closed-loop GBN setpoint experiment request for ``POST /api/experiment``."""
    if held_setpoints is None:
        held_setpoints = [setpt_n]
    if mv_channels is None or cv_channels is None:
        mv, cv = default_reactor_mapping()
        mv_channels = mv_channels or mv
        cv_channels = cv_channels or cv

    n_samples = npts // record_every + 1
    t = linspace(0.0, float(npts), n_samples)
    u = build_gbn_signal(t, base, amplitude, t_min, t_max, int(seed), channel=0)
    schedule = schedule_from_signal(u, setpt_n, record_every, base=base)

    body: dict[str, Any] = {
        "mode": "closed_loop",
        "npts": npts,
        "record_every": record_every,
        "seed": seed,
        "setpoints": {str(setpt_n): base},
        "held_setpoints": held_setpoints,
        "injections": [],
        "setpoint_schedule": schedule,
        "xmv_schedule": [],
        "loop_mode": {},
        "mv_channels": mv_channels,
        "cv_channels": cv_channels,
        "full_record": True,
        "export_stem": export_stem,
    }
    if export_dir is not None:
        body["export_dir"] = export_dir
    return body

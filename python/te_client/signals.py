"""Excitation generators aligned with mimo-sim hold-time GBN and step semantics."""

from __future__ import annotations

import random
from typing import Iterable, Sequence


def linspace(start: float, end: float, n: int) -> list[float]:
    if n <= 0:
        return []
    if n == 1:
        return [start]
    step = (end - start) / (n - 1)
    return [start + i * step for i in range(n)]


def build_step_signal(
    t: Sequence[float],
    base: float,
    steps: Iterable[tuple[float, float]],
) -> list[float]:
    """Piecewise-constant signal: at each ``(t_s, mag)`` add ``mag`` for ``t >= t_s``."""
    u = [base] * len(t)
    for t_s, mag in steps:
        for i, ti in enumerate(t):
            if ti >= t_s:
                u[i] = u[i] + mag
    return u


def build_gbn_signal(
    t: Sequence[float],
    base: float,
    amplitude: float,
    t_min: float,
    t_max: float,
    seed: int,
    channel: int = 0,
) -> list[float]:
    """Hold-time GBN: ``base ± amplitude`` with random hold in ``[t_min, t_max]`` seconds.

    Uses channel offset ``seed + 1000 + channel`` like mimo-sim ``engine.rs``.
    """
    n = len(t)
    if n == 0 or amplitude == 0.0:
        return [base] * n

    if t_max < t_min:
        t_min, t_max = t_max, t_min
    t_min = max(t_min, 1e-6)
    t_max = max(t_max, t_min)

    rng = random.Random(seed + 1000 + channel)
    t_end = t[-1]
    u = [base] * n
    level = -1.0 if rng.random() < 0.5 else 1.0
    seg_start = 0.0

    while seg_start < t_end:
        hold = rng.uniform(t_min, t_max)
        seg_end = min(seg_start + hold, t_end)
        for i, ti in enumerate(t):
            if seg_end < t_end:
                in_seg = seg_start <= ti < seg_end
            else:
                in_seg = seg_start <= ti <= seg_end
            if in_seg:
                u[i] = base + amplitude * level
        if seg_end >= t_end:
            break
        level *= -1.0
        seg_start = seg_end

    return u


def schedule_from_signal(
    values: Sequence[float],
    n: int,
    record_every: int,
    *,
    base: float | None = None,
) -> list[dict]:
    """Convert uniform grid values to TE ``setpoint_schedule`` / ``xmv_schedule`` entries."""
    base = values[0] if base is None else base
    out: list[dict] = []
    prev = base
    for k, v in enumerate(values):
        if k == 0:
            prev = v
            continue
        if abs(v - prev) > 1e-12:
            out.append(
                {
                    "start_step": k * record_every,
                    "n": n,
                    "value": float(v),
                }
            )
            prev = v
    if not out and abs(values[0] - base) > 1e-12:
        out.append({"start_step": 1, "n": n, "value": float(values[0])})
    return out

#!/usr/bin/env python3
"""Smoke test: HMPC session loop against TE (needs te-console + python_hmpc)."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2].parent / "te_client"))
sys.path.insert(0, str(Path(__file__).resolve().parents[1].parent))

from te_hmpc_bridge import ApcLoopConfig, TeHmpcLoop, load_hmpc_from_path


def build_synth_mpc(hmpc):
    import numpy as np
    from hmpc.model.state_space import HSS
    from hmpc.constraints.cv import HMPC_CV
    from hmpc.constraints.mv import HMPC_MV
    from hmpc.controller import HMPC

    ts = 60.0
    a, b, c, d = 0.95, 0.05, 1.0, 0.0
    plant = HSS(
        A=np.array([[a]]),
        B=np.array([[b]]),
        C=np.array([[c]]),
        D=np.array([[d]]),
        Ts=ts,
    )
    cv = HMPC_CV([110.0, 90.0], [130.0, 70.0], 1.0, False)
    mv = HMPC_MV([130.0, 90.0], [130.0, 90.0], 5.0, -5.0, 0.0)
    cv.Nominal = 120.0
    mv.Nominal = 120.0
    return HMPC(plant, ts=ts, ph=30, ch=15, cvs=[cv], mvs=[mv])


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--url", default="http://127.0.0.1:8787")
    p.add_argument("--steps", type=int, default=10)
    p.add_argument("--hmpc-src", type=Path, default=None)
    args = p.parse_args()

    hmpc = load_hmpc_from_path(args.hmpc_src)
    mpc = build_synth_mpc(hmpc)
    cfg = ApcLoopConfig(te_url=args.url, integrate_steps=60, steps=args.steps)
    loop = TeHmpcLoop(mpc, cfg)
    try:
        log = loop.run(cv_refs=[120.4] * args.steps)
        print(f"completed {len(log)} APC steps, last setpt={log[-1]['setpt'][17]:.3f}")
    finally:
        loop.close()


if __name__ == "__main__":
    main()

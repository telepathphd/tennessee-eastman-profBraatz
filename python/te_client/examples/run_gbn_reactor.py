#!/usr/bin/env python3
"""Run a reactor-temperature GBN setpoint experiment via te-console."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Allow running as script without install
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from te_client import TeClient, build_gbn_setpoint_experiment


def main() -> None:
    p = argparse.ArgumentParser(description="TE GBN setpoint experiment")
    p.add_argument("--url", default="http://127.0.0.1:8787")
    p.add_argument("--npts", type=int, default=7200, help="integrator steps (seconds)")
    p.add_argument("--record-every", type=int, default=60)
    p.add_argument("--seed", type=float, default=0.0)
    p.add_argument("--output", type=Path, default=Path("out"))
    p.add_argument("--stem", default="te_gbn_reactor")
    args = p.parse_args()

    args.output.mkdir(parents=True, exist_ok=True)
    body = build_gbn_setpoint_experiment(
        npts=args.npts,
        record_every=args.record_every,
        seed=args.seed,
        export_dir=str(args.output.resolve()),
        export_stem=args.stem,
    )

    client = TeClient(args.url)
    result = client.experiment(body)
    meta_path = args.output / f"{args.stem}.meta.json"
    if not meta_path.is_file() and result.get("meta_path"):
        print("server wrote:", result.get("csv_path"), result.get("meta_path"))
    print(json.dumps({k: result[k] for k in ("steps_run", "shutdown", "csv_path", "meta_path") if k in result}, indent=2))


if __name__ == "__main__":
    main()

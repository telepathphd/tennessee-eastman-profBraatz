"""Closed-loop: HMPC.move → TE Setpoint via HTTP session step API."""

from __future__ import annotations

import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Sequence

from te_client import TeClient


def load_hmpc_from_path(hmpc_src: str | Path | None = None):
    """Import ``hmpc`` from sibling ``python_hmpc`` checkout."""
    if hmpc_src is None:
        hmpc_src = Path(__file__).resolve().parents[2].parent / "HMPC_Python" / "python_hmpc" / "src"
    hmpc_src = Path(hmpc_src)
    if not hmpc_src.is_dir():
        raise ImportError(f"python_hmpc not found at {hmpc_src}")
    sys.path.insert(0, str(hmpc_src))
    import hmpc  # type: ignore

    return hmpc


@dataclass
class ApcLoopConfig:
    te_url: str = "http://127.0.0.1:8787"
    integrate_steps: int = 60
    held_setpoints: list[int] = field(default_factory=lambda: [18])
    seed: float = 0.0
    setpoint_n: int = 18
    cv_index: int = 9
    cv_meas_index: int = 9
    steps: int = 120


class TeHmpcLoop:
    """Minimal APC loop: HMPC MV → TE ``SETPT(n)``, CV ← ``XMEAS(n)``."""

    def __init__(self, mpc: Any, cfg: ApcLoopConfig) -> None:
        self.mpc = mpc
        self.cfg = cfg
        self.client = TeClient(cfg.te_url)
        self._last: dict[str, Any] | None = None

    def start(self) -> dict[str, Any]:
        sess = self.client.create_session(
            {
                "seed": self.cfg.seed,
                "integrate_steps": self.cfg.integrate_steps,
                "held_setpoints": self.cfg.held_setpoints,
                "setpoints": {str(self._sp_n): 120.4},
            }
        )
        self.session_id = sess["session_id"]
        self._last = sess["snapshot"]
        self.mpc.initial()
        return self._last

    def step_once(self, cv_ref: float | None = None) -> dict[str, Any]:
        if self.session_id is None or self._last is None:
            raise RuntimeError("call start() first")
        y = self._last["xmeas"][self._cv_idx]
        u_fb = self._last["setpt"][self._sp_n - 1]
        ysp = y if cv_ref is None else cv_ref
        self.mpc.move([y], [ysp], True, [u_fb], [])
        u_new = float(self.mpc.get_result()[0])
        self._last = self.client.session_step(self.session_id, {self._sp_n: u_new})
        return self._last

    def run(self, cv_refs: Sequence[float] | None = None, steps: int | None = None) -> list[dict[str, Any]]:
        steps = steps if steps is not None else self.cfg.steps
        self.start()
        log: list[dict[str, Any]] = []
        for k in range(steps):
            ref = cv_refs[k] if cv_refs is not None and k < len(cv_refs) else None
            log.append(self.step_once(ref))
        return log

    def close(self) -> None:
        if self.session_id:
            self.client.delete_session(self.session_id)
            self.session_id = None

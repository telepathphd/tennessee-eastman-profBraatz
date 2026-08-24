"""HTTP client for TE console experiment and session APIs."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


class TeClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8787") -> None:
        self.base_url = base_url.rstrip("/")

    def catalog(self) -> dict[str, Any]:
        return self._get("/api/catalog")

    def simulate(self, body: dict[str, Any]) -> dict[str, Any]:
        return self._post("/api/simulate", body)

    def experiment(self, body: dict[str, Any]) -> dict[str, Any]:
        return self._post("/api/experiment", body)

    def export_mimo_csv(self, time_s: list[int], mv: list[list[float]], cv: list[list[float]], record_every: int = 60) -> str:
        req = Request(
            f"{self.base_url}/api/export/mimo-csv",
            data=json.dumps(
                {
                    "time_s": time_s,
                    "mv": mv,
                    "cv": cv,
                    "record_every": record_every,
                }
            ).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urlopen(req) as resp:
            return resp.read().decode("utf-8-sig")

    def create_session(self, cfg: dict[str, Any]) -> dict[str, Any]:
        return self._post("/api/session", cfg)

    def session_step(self, session_id: str, setpoint_writes: dict[int, float]) -> dict[str, Any]:
        return self._post(
            f"/api/session/{session_id}/step",
            {"setpoint_writes": {str(k): v for k, v in setpoint_writes.items()}},
        )

    def delete_session(self, session_id: str) -> None:
        req = Request(f"{self.base_url}/api/session/{session_id}", method="DELETE")
        with urlopen(req):
            pass

    def write_experiment_csv(self, result: dict[str, Any], path: Path) -> None:
        """Write ``mv_export`` / ``cv_export`` from ``/api/experiment`` as mimo-sim CSV."""
        csv_text = self.export_mimo_csv(
            result["time_s"],
            result.get("mv_export") or [],
            result.get("cv_export") or [],
            result.get("record_every", 60),
        )
        path.write_text(csv_text, encoding="utf-8")

    def _get(self, path: str) -> dict[str, Any]:
        with urlopen(f"{self.base_url}{path}") as resp:
            return json.loads(resp.read().decode("utf-8"))

    def _post(self, path: str, body: dict[str, Any]) -> dict[str, Any]:
        req = Request(
            f"{self.base_url}{path}",
            data=json.dumps(body).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urlopen(req) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except HTTPError as err:
            payload = err.read().decode("utf-8")
            try:
                detail = json.loads(payload).get("error", payload)
            except json.JSONDecodeError:
                detail = payload
            raise RuntimeError(detail) from err
        except URLError as err:
            raise RuntimeError(f"cannot reach {self.base_url}: {err}") from err

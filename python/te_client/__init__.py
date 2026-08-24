"""Thin client: mimo-sim-style excitation + TE experiment API."""

from .client import TeClient
from .experiment import build_gbn_setpoint_experiment, default_reactor_mapping
from .signals import build_gbn_signal, build_step_signal, linspace

__all__ = [
    "TeClient",
    "build_gbn_setpoint_experiment",
    "build_gbn_signal",
    "build_step_signal",
    "default_reactor_mapping",
    "linspace",
]

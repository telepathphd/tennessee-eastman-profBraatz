//! Rust equivalent of the Fortran 77 Tennessee Eastman Process simulator
//! in this repository (`teprob.f`, `temain.f`, `temain_mod.f`).
//!
//! COMMON blocks are owned by [`TennesseeEastmanProcess`]. Indexing in the
//! public API is **1-based**, matching `XMEAS(n)`, `XMV(n)`, and `IDV(n)`.
//! Arithmetic is IEEE-754 double; it does not emulate Fortran default-kind
//! `REAL` rounding of unsuffixed literals.

pub mod catalog;
pub mod closed_loop;
pub mod open_loop;
pub mod process;
pub mod simulate;

pub use closed_loop::{ClosedLoopConfig, PlantWideController};
pub use open_loop::StripperLevelController;
pub use process::{
    default_delta_t, TennesseeEastmanProcess, DEFAULT_RNG_SEED, N_COMPONENTS, N_IDV, N_STATES,
    N_STREAMS, N_XMEAS, N_XMV, OBSERVATION_LEN,
};

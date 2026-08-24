# IEEE-754 double, not Fortran REAL rounding

The replica used to round unsuffixed literals through `f32` so that `TEINIT` / `TEFUNC` bit-matched gfortran’s default-kind `REAL` constants stored in `DOUBLE PRECISION`. That couples the model to a compiler quirk rather than the written equations. Arithmetic is now `f64` throughout: constants and mixed-mode expressions (`DELTAT = 1/3600`, Arrhenius `E/R/T`, Kelvin offsets) are evaluated as written. Trajectories will not match Fortran dumps; tests check the published base-case operating point and integrator/controller behaviour instead.

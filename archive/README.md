# Historical Fortran 77

This directory is the original Illinois / Braatz distribution: Fortran 77
sources and the published FDD training/testing files. It is kept as a
reference for the replica in `rust/` and the operator console in `web/`.
It is not the live implementation.

The original package `readme.txt` is preserved next to the sources.

# Author

> Copyright (c) 1998-2002 The Board of Trustees of the University of Illinois All rights reserved.
>
> Developed by: Large Scale Systems Research Laboratory
>
> Professor Richard Braatz, Director
> Department of Chemical Engineering
> University of Illinois
>
> http://brahms.scs.uiuc.edu

# Contents

This directory contains the Fortran 77 codes for the open-loop and the closed-loop simulations for the Tennessee Eastman process (TEP) as well as the training and testing data files used for evaluating the data-driven methods (PCA, PLS, FDA, and CVA).

File name | Description
--------- | -----------
[`temain.f`](#temainf) | open loop simulation codes for the TEP
[`temain_mod.f`](#temainmodf) | closed loop simulation codes for the TEP
[`teprob.f`](#teprobf) | subprogram for the simulation codes for the TEP
`d00.dat` | training file for the normal operating conditions
`d00_te.dat` | testing file for the normal operating conditions
`d01.dat` | training file for Fault 1
`d01_te.dat`| testing file for Fault 1
`d02.dat` | training file for Fault 2
`d02_te.dat`| testing file for Fault 2
`d21.dat`|training file for Fault 21
`d21_te.dat`| testing file for Fault 21

Each training data file contains 480 rows and 52 columns and each testing data file contains 960 rows and 52 columns.
An observation vector at a particular time instant is given by
```fortran
 x = [XMEAS(1), XMEAS(2), ..., XMEAS(41), XMV(1), ..., XMV(11)]^T
```
where `XMEAS(n)`is the n-th measured variable and `XMV(n)` is the n-th manipulated variable.

The measurement, valve, and disturbance tables that the replica uses are in the [repository README](../README.md). The lists below stay with the original sources.

---

## `temain.f`

Main program for demonstrating application of the **Tennessee Eastman Process Control Test Problem**.

> James J. Downs and Ernest F. Vogel
>
> Process and Control Systems Engineering
>
> Tennessee Eastman Company
>
> P.O. Box 511
>
> Kingsport, TN  37662

**Reference**

- *A Plant-Wide Industrial Process Control Problem*, Presented at the AIChE 1990 Annual Meeting Industrial Challenge Problems in Process Control, Paper #24a. Chicago, Illinois, November 14, 1990.
- [*A Plant-Wide Industrial Process Control Problem*, Computers and Chemical Engineering, Vol. 17, No. 3, pp. 245-255 (1993)](https://doi.org/10.1016/0098-1354(93)80018-I).

## `temain_mod.f`

Main program for demonstrating application of the modified Tennessee Eastman Process Control Test Problem.

This *new version* is a **closed-loop plant-wide control scheme** for the **Tennessee Eastman Process Control Test Problem**.
The modifications are by:

> Evan L. Russell, Leo H. Chiang and Richard D. Braatz
>
>  Large Scale Systems Research Laboratory
>
>  Department of Chemical Engineering
>
>  University of Illinois at Urbana-Champaign
>
>  600 South Mathews Avenue, Box C-3
>
>  Urbana, Illinois 61801
>
>  http://brahms.scs.uiuc.edu

Original codes of the **Tennessee Eastman Process Control Test Problem** written by:

> James J. Downs and Ernest F. Vogel
>
> Process and Control Systems Engineering
>
> Tennessee Eastman Company
>
> P.O. Box 511
>
> Kingsport, Tennessee 37662

### License

The modified text is Copyright 1998-2002 by The Board of Trustees of the University of Illinois. All rights reserved.

Permission hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal with the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimers.
2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimers in the documentation and/or other materials provided with the distribution.
3. Neither the names of Large Scale Research Systems Laboratory, University of Illinois, nor the names of its contributors may be used to endorse or promote products derived from this Software without specific prior written permission.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE CONTRIBUTORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Users should cite the *original code* using the following references:

- J.J. Downs and E.F. Vogel, *A plant-wide industrial process control problem*. Presented at the AIChE 1990 Annual Meeting, Session on Industrial Challenge Problems in Process Control, Paper #24a Chicago, Illinois, November 14, 1990.
- [J.J. Downs and E.F. Vogel, *A plant-wide industrial process control problem*, Computers and Chemical Engineering, 17:245-255 (1993)](https://doi.org/10.1016/0098-1354(93)80018-I).

Users should cite the *modified code* using the following references:

- [E.L. Russell, L.H. Chiang, and R.D. Braatz. Data-driven Techniques for Fault Detection and Diagnosis in Chemical Processes, Springer-Verlag, London, 2000](https://doi.org/10.1007/978-1-4471-0409-4).
- [L.H. Chiang, E.L. Russell, and R.D. Braatz. Fault Detection and Diagnosis in Industrial Systems, Springer-Verlag, London, 2001](https://doi.org/10.1007/978-1-4471-0347-9).
- [L.H. Chiang, E.L. Russell, and R.D. Braatz. *Fault diagnosis in chemical processes using Fisher discriminant analysis, discriminant partial least squares, and principal component analysis*, Chemometrics and Intelligent Laboratory Systems, 50:243-252, 2000](https://doi.org/10.1016/S0169-7439(99)00061-1).
- [E.L. Russell, L.H. Chiang, and R.D. Braatz. *Fault detection in industrial processes using canonical variate analysis and dynamic principal component analysis*, Chemometrics and Intelligent Laboratory Systems, 51:81-93, 2000](https://doi.org/10.1016/S0169-7439(00)00058-7).

### Instructions for running the program

1. Go to line `220`, change `NPTS` to the number of data points to simulate. For each minute of operation, 60 points are generated.
2. Go to line `226`, change `SSPTS` to the number of data points to simulate in steady state operation before implementing the disturbance.
3. Go to line `367`, implement any of the 21 programmed disturbances. For example, to implement disturbance 2, type `IDV(2)=1`.
4. The program will generate 15 output files and all data are recorded every 180 seconds, see Table 1 for details.

	The default path is the home directory.	To change the file name and path, modify lines `346-360` accordingly.

	To overwrite the files that already existed, change `STATUS='new'` to `STATUS='old'` from lines `346-360`.

	**Table 1:** Content of the output files

    \#| File Name | Content
    --| --------- | -------
     1| `TE_data_inc.dat` | Time (in seconds)
     2| `TE_data_mv1.dat` | Measurements for manipulated variables 1 to 4
     3| `TE_data_mv2.dat` | Measurements for manipulated variables 5 to 8
     4| `TE_data_mv3.dat` | Measurements for manipulated variables 9 to 12
     5| `TE_data_me01.dat` | Measurements for measurement variables 1 to 4
     6| `TE_data_me02.dat` | Measurements for measurement variables 5 to 8
     7| `TE_data_me03.dat` | Measurements for measurement variables 9 to 12
     8| `TE_data_me04.dat` | Measurements for measurement variables 13 to 16
     9| `TE_data_me05.dat` | Measurements for measurement variables 17 to 20
    10| `TE_data_me06.dat` | Measurements for measurement variables 21 to 24
    11| `TE_data_me07.dat` | Measurements for measurement variables 25 to 28
    12| `TE_data_me08.dat` | Measurements for measurement variables 29 to 32
    13| `TE_data_me09.dat` | Measurements for measurement variables 33 to 36
    14| `TE_data_me10.dat` | Measurements for measurement variables 37 to 40
    15| `TE_data_me11.dat` | Measurements for measurement variable 41

5. To ensure the randomness of the measurement noises, the random number `G` in the sub program (`teprob.f`, line 1187) has to be changed each time before running `temain_mod.f`.
6. Save the changes in `temain_mod.f` and `teprob.f` and compile the program in unix by typing
	```bash
	  f77 temain_mod.f teprob.f
	```
7. Run the program by typing
	```bash
	  a.out
	```

The replica CLI (`cargo run --release --bin temain_mod` in `rust/`) maps these edit points to flags instead of source edits.

## `teprob.f`

Revised 4-4-91 to correct error in documentation of manipulated variables

**Tennessee Eastman Process Control Test Problem**

> James J. Downs and Ernest F. Vogel
>
> Process and Control Systems Engineering
>
> Tennessee Eastman Company
>
> P.O. Box 511
>
> Kingsport, TN  37662

**Reference**

- A Plant-Wide Industrial Process Control Problem". Presented at the AIChE 1990 Annual Meeting Industrial Challenge Problems in Process Control, Paper #24a Chicago, Illinois, November 14, 1990.

### Subroutines

- `TEFUNC` - Function evaluator to be called by integrator
- `TEINIT` - Initialization
- `TESUBi` - Utility subroutines ($i = 1, 2, ..., 8$)

The process simulation has 50 states (`NN=50`).

If the user wishes to integrate additional states, `NN` must be increased accordingly in the calling program.

The additional states should be appended to the end of the `YY` vector, e.g. `YY(51), ...`. The additional derivatives should be appended to the end of the `YP` vector, e.g. `YP(51),...`.

To initialize the new states and to calculate derivatives for them, we suggest creating new function evaluator and initialization routines as follows.

```fortran
C-----------------------------------------------
C
      SUBROUTINE FUNC(NN,TIME,YY,YP)
C
      INTEGER NN
      DOUBLE PRECISION TIME, YY(NN), YP(NN)
C
C  Call the function evaluator for the process
C
      CALL TEFUNC(NN,TIME,YY,YP)
C
C  Calculate derivatives for additional states
C
      YP(51) = ....
      YP(52) = ....
         .
         .
         .
      YP(NN) = ....
C
      RETURN
      END
C
C-----------------------------------------------
C
      SUBROUTINE INIT(NN,TIME,YY,YP)
C
      INTEGER NN
      DOUBLE PRECISION TIME, YY(NN), YP(NN)
C
C  Call the initialization for the process
C
      CALL TEINIT(NN,TIME,YY,YP)
C
C  Initialize additional states
C
      YY(51) = ....
      YY(52) = ....
         .
         .
         .
      YY(NN) = ....
C
      RETURN
      END
C
C-----------------------------------------------
```

*Differences between the code and its description in the paper:*

1. Subroutine `TEINIT` has `TIME` in the argument list. `TEINIT` sets `TIME` to zero.
2. There are 8 utility subroutines (`TESUBi`) rather than 5.
3. Process disturbances 14 through 20 do *NOT* need to be used in conjunction with another disturbance as stated in the paper. All disturbances can be used alone or in any combination.

The published names for `IDV(16..20)` stay **Unknown** for fault-detection benchmarks; `TEFUNC` still wires hidden mechanisms (stripper steam, heat removal, valve sticking, reactor outlet flow). `d21.dat` is a dataset file, not a twenty-first `IDV` flag.

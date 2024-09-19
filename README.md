# castep-auto-hubbard

## Requirements

1. non conserving or ultrasoft
   - modify SPECIES_POT section
1. cutoff energy
1. energy tolerance criteria option, e.g. 1e-3
1. for cluster, modify kpoint settings by KPOINT_MP_GRID
   - substitute KPOINTS_LIST to KPOINTS_MP_GRID
1. submission command
1. mode: serial or parallel
1. arch detect
1. qsub nodes number setting
1. read data supports more atoms
1. Manipulate more atoms
1. Skip if castep exists AND finalisation time detected

### another script for non-converged cases

1. energy tolerance
2. dm to edft (currently not supported)

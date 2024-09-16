#! /bin/bash
# Automatic hubbard_U increment calculation workflow
# !!!! Caution: substitute function `faux_castep_run` by "$castep_command"
#
# Please read comments in this script if you want to understand and/or
# modify the necessary parts for actual needs.

# 1. Setup before
init_u=0.000000010000000
init_elec_energy_tol=1e-5
# !!! Please adjust this variable to the actual command to
# start castep calculation.
# Example:
# castep_command="mpirun -np 4 castep_alphaOverU.mpi" (standalone)
# castep_command="qsub hpc.pbs.sh" (standalone)
castep_command_u="faux_castep_run"
castep_command_alpha="faux_castep_run"
# castep_command_u="qsub hpc.pbs.sh"
castep_command_alpha="qsub hpc.pbs.sh"
source "$(dirname "$0")"/functions_linux.sh

if [[ $1 == '' ]]; then
	read -r -e -p "seed file folder path:" SEED_PATH
else
	SEED_PATH=$1
fi

job_type=$2

main "$init_u" "$init_elec_energy_tol" 2 12 "$job_type"

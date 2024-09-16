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
source "$(dirname "$0")"/functions_linux.sh

if [[ $1 == '' ]]; then
	read -r -e -p "seed file folder path:" SEED_PATH
else
	SEED_PATH=$1
fi

job_type=$2

# Maximum parallel jobs "$N"
N=32
parallel "$init_u" "$init_elec_energy_tol" 2 12 "$job_type" $N

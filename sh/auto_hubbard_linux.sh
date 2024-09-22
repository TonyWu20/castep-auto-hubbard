#! /bin/bash
# Automatic hubbard_U increment calculation workflow
# !!!! Caution: substitute function `faux_castep_run` by "$castep_command"
#
# Please read comments in this script if you want to understand and/or
# modify the necessary parts for actual needs.

# 1. Setup before
init_u=0.000000010000000
init_elec_energy_tol=1e-5
U_increment=2
U_final=12
# !!! Please adjust this variable to the actual command to
# start castep calculation.
castep_command_u="qsub hpc.pbs_AU.sh"
castep_command_alpha="qsub hpc.pbs_HU.sh"
castep_command_u="faux_castep_run GDY_111_Fe_U"
castep_command_alpha="faux_castep_run GDY_111_Fe_U"

source "$(dirname "$0")"/functions_linux.sh

# First argument is seed file folder path
if [[ $1 == '' ]]; then
	read -r -e -p "seed file folder path:" SEED_PATH
else
	SEED_PATH=$1
fi

# Second argument is job type: u or alpha
# Will be reconfirmed for blank or invalid input
job_type=$2

# Third argument is number of perturbations
if [[ $3 == '' ]]; then
	read -r -e -p "Perturb times (default to 5): " PERTURB_TIMES
	PERTURB_TIMES=${PERTURB_TIMES:-5}
	echo "Number of perturbation: $PERTURB_TIMES"
else
	PERTURB_TIMES=$3
fi

# Fourth argument is the increment of Hubbard_alpha
if [[ $4 == '' ]]; then
	read -r -e -p "Perturb increment (e.g. +0.05/per step): " PERTURB_INCREMENT
	PERTURB_INCREMENT=${PERTURB_INCREMENT:-0.05}
	echo "Increment of alpha: $PERTURB_INCREMENT"
else
	PERTURB_INCREMENT=$4
fi

# Fifth argument is the initial U
if [[ $5 == '' ]]; then
	read -r -e -p "Init U (non zero, default to 0.000000010000000): " init_u
elif [[ $5 =~ ^[+-]?[0-9]+\.?[0-9]*$ ]]; then
	init_u=$5
else
	echo "Input init U is not a valid float number; run by default 0.000000010000000"
fi

# List of running modes for selection
RUN_MODES=(serial parallel read)

# Sixth  argument is running mode
if [[ $6 == '' ]]; then
	PS3="Please choose running mode (enter number): "
	select choice in "${RUN_MODES[@]}"; do
		case $choice in
		serial | parallel)
			run_mode="$choice"
			break
			;;
		*) echo "Invalid option $choice" ;;
		esac
	done
elif [[ $6 == 'serial' || $6 == 'parallel' || $6 == 'read' ]]; then
	run_mode=$6
else
	echo "Invalid input of running mode (serial/parallel/read), please restart the program"
	exit
fi

# PERTURB_TIMES=5
# PERTURB_INCREMENT=0.05
#

setup "$init_u" "$init_elec_energy_tol" "$U_increment" "$U_final" "$job_type"
setup_perturbation "$PERTURB_TIMES" "$PERTURB_INCREMENT"
setup_castep_command "$castep_command_u" "$castep_command_alpha"

N=32

case $run_mode in
serial) serial ;;
parallel) parallel $N ;;
read) after_read ;;
*) exit ;;
esac

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
job_type_arg=$2
job_type_input "$job_type_arg"

# List of running modes for selection
RUN_MODES=(serial parallel read)
# Third  argument is running mode
if [[ $3 == '' ]]; then
	PS3="Please choose running mode (enter number): "
	select choice in "${RUN_MODES[@]}"; do
		case $choice in
		serial | parallel | read)
			run_mode="$choice"
			break
			;;
		*) echo "Invalid option $choice" ;;
		esac
	done
elif [[ $3 == 'serial' || $3 == 'parallel' || $3 == 'read' ]]; then
	run_mode=$3
else
	echo "Invalid input of running mode (serial/parallel/read), please restart the program"
	exit
fi

case $run_mode in
serial | parallel)
	# Fourth argument is the initial alpha value shift in perturbation
	if [[ $4 == '' ]]; then
		read -r -e -p "Perturb initial Δalpha (default to 0.05): " init_alpha
		PERTURB_INIT_ALPHA=${init_alpha:-0.05}
	elif [[ $4 =~ ^[+-]?[0-9]+\.?[0-9]*$ ]]; then
		PERTURB_INIT_ALPHA=$4
	else
		echo "Input init Δalpha is not a valid float number; run by default 0.05"
	fi

	# Fifth argument is the increment of Hubbard_alpha
	if [[ $5 == '' ]]; then
		read -r -e -p "Perturb increment (e.g. +0.05/per step): " increment
		PERTURB_INCREMENT=${increment:-0.05}
		echo "Increment of alpha: $PERTURB_INCREMENT"
	else
		PERTURB_INCREMENT=$5
	fi

	# Fifth argument is the initial U
	if [[ $6 == '' ]]; then
		read -r -e -p "Perturb final Δalpha (default to 0.25): " final_alpha
		PERTURB_FINAL_ALPHA=${final_alpha:-0.25}
	elif [[ $6 =~ ^[+-]?[0-9]+\.?[0-9]*$ ]]; then
		PERTURB_FINAL_ALPHA=$6
	else
		echo "Input final Δalpha is not a valid float number; run by default 0.25"
	fi

	PERTURB_TIMES=$(seq "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA" | wc -l)
	echo "Init Δalpha=$PERTURB_INIT_ALPHA; increment=$PERTURB_INCREMENT; final Δalpha=$PERTURB_FINAL_ALPHA"
	echo -e "Perturbation times: $PERTURB_TIMES\n"
	setup "$init_u" "$init_elec_energy_tol" "$U_increment" "$U_final" "$job_type"
	setup_perturbation "$PERTURB_INIT_ALPHA" "$PERTURB_INCREMENT" "$PERTURB_FINAL_ALPHA"
	setup_castep_command "$castep_command_u" "$castep_command_alpha"

	N=32
	case $run_mode in
	serial) serial ;;
	parallel) parallel $N ;;
	*) exit ;;
	esac
	;;
read)
	if [[ $4 == '' ]]; then
		read -r -e -p "How many times of perturbation did you set? " PERTURB_TIMES
	else
		PERTURB_TIMES=$4
	fi
	setup "$init_u" "$init_elec_energy_tol" "$U_increment" "$U_final" "$job_type"
	after_read
	;;
*) exit ;;
esac

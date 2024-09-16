#! /bin/bash
# Automatic hubbard_U increment calculation workflow
# !!!! Caution: substitute function `faux_castep_run` by "$castep_command"
#
# Please read comments in this script if you want to understand and/or
# modify the necessary parts for actual needs.

if [[ $1 == '' ]]; then
	read -r -e -p "seed file folder path:" SEED_PATH
else
	SEED_PATH=$1
fi
# 1. Setup before
init_u=0.000000010000000
init_elec_energy_tol=1e-5
# !!! Please adjust this variable to the actual command to
# start castep calculation.
# Example:
# castep_command="mpirun -np 4 castep_alphaOverU.mpi"
castep_command="faux_castep_run"

function faux_castep_run {
	sleep 3
	touch "$1.castep"
	sleep 3
	{
		echo "           1           1 Total:    4.88454712510949       Mz:"
		echo "           1           2 Total:    2.04480063601341       Mz:"
		echo "           1           1 Total:    4.88222767868911       Mz:"
		echo "           1           2 Total:    2.03625090967629       Mz:"
		echo "           1           1 Total:    4.88417863385162       Mz:"
		echo "           1           2 Total:    2.03022846329140       Mz:"
	} >>"$1.castep"
	sleep 2
	echo "Finalisation time" >>"$1.castep"
}

function hubbard_u {
	local i=$1
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i "s/d:.*/d: $u_value/g" "$cell_file"
	echo "Initiate U to $u_value"
	printf "\n" >>"$cell_file"
	cat "$cell_file" >"$cell_file".bak
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file" | awk '{sub(/:.*/, u_value)gsub(/_U/, "_ALPHA")}1' u_value=": $init_u" >>"$cell_file".bak
	mv "$cell_file".bak "$cell_file"
}

# Usage: cell_before cell_file_path U_value
function cell_before {
	local cell_file=$1
	local i=$2
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i $"s/\r$//" "$cell_file"
	hubbard_u "$i"
}

function param_before_perturb {
	local param_file=$1
	# remove \r from Windows generated files
	sed -i "s/\r//" "$param_file"
	sed -i '/^task.*/a !continuation : default\\niprint=3' "$param_file"
	sed -i "s/\(elec_energy_tol :\).*/\1 $init_elec_energy_tol/" "$param_file"
	sed -i '/^fine_grid_scale.*/d' "$param_file"
	sed -i -E "s/(grid_scale)[ :]+[0-9.]+/\1 : 1.750000000000000/" "$param_file"
}

function setup_before_perturb {
	local i=$1
	local folder_name=U_"$i"_u
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	# create new folder U_x
	echo "create $folder_name"
	mkdir -p "$folder_name"
	# copy files without '.castep' to U_x
	echo "copy files"
	find . -maxdepth 1 -type f -not -name "*.castep" -print0 | xargs -0 -I {} cp {} "$folder_name"
	local cell_file
	cell_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.cell")
	# setup cell
	cell_before "$cell_file" "$i"
	# setup param
	local param_file
	param_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.param")
	param_before_perturb "$param_file"
	# run?
	# return new folder_name
	setup_init_folder="$folder_name"
}

function param_after_perturb {
	local param_file=$1
	# remove "!" before continuation:default
	sed -i "s/^!//" "$param_file"
	# Divide elec_energy_tol by 10 to 1e-6
	sed -i -E "s/(elec_energy_tol :).*/\1 1e-6/" "$param_file"
}

function cell_after_perturb {
	local cell_file=$1
	local value
	value=$(awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file" | awk 'NR==2 {print $4}')
	local after_value
	after_value=$(echo "$value" 0.05 | awk '{printf "%.14f0", $1+$2}')
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/ {sub(/d: .*/, a)}1' a="d: $after_value" "$cell_file" >"$cell_file".bak
	mv "$cell_file".bak "$cell_file"
}

function setup_after_perturb {
	local u_i=$1
	local step=$2
	local folder_name=$3
	local new_folder_name="$folder_name""_$step"
	local dest="$folder_name/$new_folder_name"
	mkdir -p "$dest"
	find ./"$folder_name" -maxdepth 1 -type f -not -name "*.castep" -print0 | xargs -0 -I {} cp {} "$dest"
	local u_value
	u_value=$(echo "$init_u $u_i $step" | awk '{printf "%.14f0", $1+$2+$3}')
	local param_file
	param_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.param")
	# setup param after perturbation
	param_after_perturb "$param_file"
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	# setup cell after perturbation
	cell_after_perturb "$cell_file"
	# return new folder name
	setup_next_folder=$dest
}

function start_job {
	local current_dir
	current_dir=$(pwd)
	local job_dir=$1
	local job_name
	job_name=$(find ./"$job_dir" -maxdepth 1 -type f -name "*.cell" | awk -F / '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	cd "$job_dir" || exit
	# Here is the command to start calculation
	# Use a single & to move the job to background
	$castep_command "$job_name" >/dev/null &
	cd "$current_dir" || exit
}

function monitor_job_done {
	local dest=$1
	# get job name by extracting filestem
	# find under destination
	local jobname
	jobname=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell" | awk '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	local castep_file="$jobname.castep"
	# Wait for generation of `.castep` file
	until [ -f "$castep_file" ]; do
		printf "Waiting for generation of %s\r" "$castep_file"
		sleep 1
	done
	echo -e "\nFound: $castep_file"
	# Monitor appearance of "Finalised time"
	local count
	count=$(grep -c "Finalisation time" "$castep_file")
	while [ "$count" -lt 1 ]; do
		printf "Waiting for job completion...\r"
		count=$(grep -c "Finalisation time" "$castep_file")
		sleep 1
	done
	echo -e "\nCalculation completed!"
	finished_castep_file="$castep_file"
	finished_job_name="$jobname"
	# setup after perturb
	read_data
}

function read_data {
	local castep_file=$finished_castep_file
	local grep_result_1
	grep_result_1=$(grep -Ei "[[:blank:]]+1[[:blank:]]+1 Total" "$castep_file")
	local data_1_before_scf
	data_1_before_scf=$(echo "$grep_result_1" | awk 'NR==1 {print $4}')
	local data_1_scf_1st
	data_1_scf_1st=$(echo "$grep_result_1" | awk 'NR==2 {print $4}')
	local data_1_scf_last
	data_1_scf_last=$(echo "$grep_result_1" | tail -n 1 | awk '{print $4}')
	local grep_result_2
	grep_result_2=$(grep -Ei "[[:blank:]]+1[[:blank:]]+2 Total" "$castep_file")
	local data_2_before_scf
	data_2_before_scf=$(echo "$grep_result_2" | awk 'NR==1 {print $4}')
	local data_2_scf_1st
	data_2_scf_1st=$(echo "$grep_result_2" | awk 'NR==2 {print $4}')
	local data_2_scf_last
	data_2_scf_last=$(echo "$grep_result_2" | tail -n 1 | awk '{print $4}')
	printf "%s, %f, %f, %f\n" "$finished_job_name" "$data_1_before_scf" "$data_1_scf_1st" "$data_1_scf_last" >>result.csv
	printf "%s, %f, %f, %f\n" "$finished_job_name" "$data_2_before_scf" "$data_2_scf_1st" "$data_2_scf_last" >>result.csv
}

# Maximum parallel jobs "$N"
N=4
function main {
	cd "$SEED_PATH" || exit
	printf "Jobname, Before SCF, 1st SCF, Last SCF\n" >result_u.csv
	for i in $(seq 0 "$2" "$1"); do
		(
			# .. do your stuff here
			setup_before_perturb "$i"
			init_folder="$setup_init_folder"
			# run castep
			# castep $SEED_PATH/$SEED_NAME
			# monitor result
			start_job "$init_folder"
			monitor_job_done "$init_folder"
			# echo  "Setup next perturbation step\r"
			setup_after_perturb "$i" 1 "$init_folder"
			next_folder=$setup_next_folder
			start_job "$next_folder"
			monitor_job_done "$next_folder"
		) &

		# allow to execute up to $N jobs in parallel
		if [[ $(jobs -r -p | wc -l) -ge $N ]]; then
			# now there are $N jobs already running, so wait here for any job
			# to be finished so there is a place to start next one.
			wait -n
		fi

	done

	# no more jobs to be started but wait for pending jobs
	# (all need to be finished)
	wait

	echo "all done"
}

# Run the parallel program.
# 1st integer after `main` is the upper limit of U (U <=number)
# 2nd integer is the increment step of U
# Example: `main 12 2` runs with Us of 0, 2, 4, 6, 8, 10, 12.
main 12 2

#! /bin/bash
# Automatic hubbard_U increment calculation workflow
# Run under

if [[ $1 == '' ]]; then
	read -r -e -p "seed file folder path:" SEED_PATH
else
	SEED_PATH=$1
fi
cd "$SEED_PATH" || exit

# 1. Setup before
init_u=0.000000010000000
init_elec_energy_tol=1e-5

function cell_before {
	local cell_file=$1
	local i=$2
	sed -i '' $"s/\r$//" "$cell_file"
	sed -i '' "s/d: 0.*/d: $init_u/g" "$cell_file"
	echo "Initiate U to 0"
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	printf "\n" >>"$cell_file"
	cat "$cell_file" >"$cell_file".bak
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file" | awk '{sub(/:.*/, u_value)gsub(/_U/, "_ALPHA")}1' u_value=": $u_value" >>"$cell_file".bak
	mv "$cell_file".bak "$cell_file"
}

function param_before_perturb {
	local param_file=$1
	# remove \r from Windows generated files
	sed -i '' $"s/\r$//" "$param_file"
	sed <"$param_file" '/^task.*/a \ 
!continuation : default \
iprint=3 \
' | sed "s/\(elec_energy_tol :\).*/\1 $init_elec_energy_tol/" | sed '/^fine_grid_scale.*/d' | sed -E "s/(grid_scale)[ :]+[0-9.]+/\1 : 1.750000000000000/" >"$param_file".bak
	mv "$param_file".bak "$param_file"
}

function setup_before_perturb {
	local i=$1
	local folder_name=U_$i
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	# create new folder U_x
	echo "create $folder_name"
	mkdir -p "$folder_name"
	# copy files without '.castep' to U_x
	echo "copy files"
	find . -d 1 -type f -not -name "*.castep" -print0 | xargs -0 -I {} cp {} "$folder_name"
	local cell_file
	cell_file=$(find ./"$folder_name" -d 1 -type f -name "*.cell")
	# setup cell
	cell_before "$cell_file" "$i"
	# setup param
	local param_file
	param_file=$(find ./"$folder_name" -type f -name "*.param" -d 1)
	param_before_perturb "$param_file"
	# run?
	# return new folder_name
	setup_init_folder="$folder_name"
}

function param_after_perturb {
	local param_file=$1
	# remove "!" before continuation:default
	sed -i '' "s/^!//" "$param_file"
	# Divide elec_energy_tol by 10 to 1e-6
	sed -i '' -E "s/(elec_energy_tol :).*/\1 1e-6/" "$param_file"
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
	find ./"$folder_name" -d 1 -type f -not -name "*.castep" -print0 | xargs -0 -I {} cp {} "$dest"
	local u_value
	u_value=$(echo "$init_u $u_i $step" | awk '{printf "%.14f0", $1+$2+$3}')
	local param_file
	param_file=$(find ./"$dest" -type f -d 1 -name "*.param")
	# setup param after perturbation
	param_after_perturb "$param_file"
	local cell_file
	cell_file=$(find ./"$dest" -type f -d 1 -name "*.cell")
	# setup cell after perturbation
	cell_after_perturb "$cell_file"
	# return new folder name
	setup_next_folder=$dest
}

function monitor_job_done {
	local dest=$1
	# get job name by extracting filestem
	# find under destination
	local jobname
	jobname=$(find ./"$dest" -d 1 -type f -name "*.cell" | awk '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
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
	echo "Calculation completed!"
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

function main {
	local i=$1
	setup_before_perturb "$i"
	init_folder="$setup_init_folder"
	# run castep
	# castep $SEED_PATH/$SEED_NAME
	# monitor result
	printf "Jobname, Before SCF, 1st SCF, Last SCF\n" >result.csv
	monitor_job_done "$init_folder"
	# echo  "Setup next perturbation step\r"
	setup_after_perturb "$i" "$init_folder"
	next_folder=$setup_next_folder
	monitor_job_done "$next_folder"
}

for ((i = 0; i < 4; i += 2)); do
	main $i &
done
wait

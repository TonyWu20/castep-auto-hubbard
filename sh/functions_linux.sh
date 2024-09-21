# castep_command_u="qsub hpc.pbs_AU.sh"
# castep_command_alpha="qsub hpc.pbs_HU.sh"

function job_type_input {
	if [[ $1 == '' ]]; then
		read -r -e -p "job type: u/alpha?" job_type
	else
		job_type=$1
	fi

	until [[ $job_type == 'u' || $job_type == 'U' || $job_type == 'alpha' || $job_type == 'Alpha' ]]; do
		echo "Invalid job_type input: $job_type"
		read -r -e -p "Please input job type correctly as follows: u/U or alpha/Alpha?" job_type
	done
	case $job_type in
	u | U) job_type="u" ;;
	alpha | Alpha) job_type="alpha" ;;
	*)
		echo "Invalid job type input; ends program"
		exit
		;;
	esac

	input_job_type="$job_type"
}

function faux_castep_run {
	sleep 1
	touch "$1.castep"
	sleep 1
	{
		echo "           1           1 Total:    4.88454712510949       Mz:"
		echo "           1           2 Total:    2.04480063601341       Mz:"
		echo "           1           1 Total:    4.88222767868911       Mz:"
		echo "           1           2 Total:    2.03625090967629       Mz:"
		echo "           1           1 Total:    4.88417863385162       Mz:"
		echo "           1           2 Total:    2.03022846329140       Mz:"
	} >>"$1.castep"
	sleep 1
	echo "Finalisation time" >>"$1.castep"
}

function hubbard_u {
	local init_u=$1
	local i=$2
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i -E "s/([spdf]):.*/\1: $u_value/g" "$cell_file"
	echo "Initiate U to $u_value"
	printf "\n" >>"$cell_file"
	cat "$cell_file" >"$cell_file".bak
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file" | awk '{sub(/:.*/, u_value)gsub(/_U/, "_ALPHA")}1' u_value=": $init_u" >>"$cell_file".bak
	echo "Initiate Alpha to $init_u"
	mv "$cell_file".bak "$cell_file"
}

function hubbard_alpha {
	local init_u=$1
	local i=$2
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i -E "s/([spdf]):.*/\1: $init_u/g" "$cell_file"
	echo "Initiate U to $init_u"
	printf "\n" >>"$cell_file"
	cat "$cell_file" >"$cell_file".bak
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file" | awk '{sub(/:.*/, u_value)gsub(/_U/, "_ALPHA")}1' u_value=": $u_value" >>"$cell_file".bak
	echo "Inititate Alpha to $u_value"
	mv "$cell_file".bak "$cell_file"
}

function cell_before {
	local cell_file=$1
	local init_u=$2
	local i=$3
	local job_type=$4
	local hubbard_set
	case $job_type in
	U | u) hubbard_set=hubbard_u ;;
	alpha | Alpha) hubbard_set=hubbard_alpha ;;
	*) exit ;;
	esac
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	sed -i "s/\r//" "$cell_file"
	"$hubbard_set" "$init_u" "$i"
	echo -e "---------------------------------------------------------------------\nFor $cell_file:"
	awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/' "$cell_file"
	echo -e "\n"
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file"
	echo "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}

function param_before_perturb {
	local param_file=$1
	local init_elec_energy_tol=$2
	# remove \r from Windows generated files
	sed -i "s/\r//" "$param_file"
	sed -i '/^task.*/a !continuation : default\niprint=3' "$param_file"
	sed -i "s/\(elec_energy_tol :\).*/\1 $init_elec_energy_tol/" "$param_file"
	sed -i '/^fine_grid_scale.*/d' "$param_file"
	sed -i -E "s/(grid_scale)[ :]+[0-9.]+/\1 : 1.750000000000000/" "$param_file"
}

function setup_before_perturb {
	local init_u=$1
	local i=$2
	local init_elec_energy_tol=$3
	local job_type=$4
	local folder_name=U_"$i"_"$job_type"
	local u_value
	u_value=$(echo "$init_u $i" | awk '{printf "%.14f0", $1+$2}')
	# create new folder U_x
	echo "create $folder_name"
	mkdir -p "$folder_name"
	# copy files without '.castep' to U_x
	echo "copy files"
	find . -maxdepth 1 -type f -not -name "*.castep" -not -name "*.txt" -not -name "*.csv" -print0 | xargs -0 -I {} cp {} "$folder_name"
	local cell_file
	cell_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.cell")
	# setup cell
	cell_before "$cell_file" "$init_u" "$i" "$job_type"
	# setup param
	local param_file
	param_file=$(find ./"$folder_name" -maxdepth 1 -type f -name "*.param")
	param_before_perturb "$param_file" "$init_elec_energy_tol"
	# run?
	# return new folder_name
	setup_init_folder="$folder_name"
}

function param_after_perturb {
	local param_file=$1
	# remove "!" before continuation:default
	sed -i "s/^!//" "$param_file"
	# Divide elec_energy_tol by 10 to 1e-6
	local new_elec_energy_tol
	new_elec_energy_tol=$(echo "$init_elec_energy_tol" 10 | awk '{printf "%e", $1/$2}')
	sed -i -E "s/(elec_energy_tol :).*/\1 $new_elec_energy_tol/" "$param_file"
}

function cell_after_perturb {
	local cell_file=$1
	local perturb_step=$2
	local perturb_increment=$3
	local value
	value=$(awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file" | awk 'NR==2 {print $4}')
	local after_value
	after_value=$(echo "$value" "$perturb_increment" "$perturb_step" | awk '{printf "%.14f0", $1+$2*$3}')
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/ {sub(/: .*/, a)}1' a=": $after_value" "$cell_file" >"$cell_file".bak
	echo "---------------------------------------------------------------------"
	echo -e "$cell_file\nPerturbation count: $perturb_step\nUpdate alpha to $after_value"
	awk '/%BLOCK HUBBARD_ALPHA/,/%ENDBLOCK HUBBARD_ALPHA/' "$cell_file.bak"
	echo "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
	mv "$cell_file".bak "$cell_file"
}

function setup_after_perturb {
	local perturb_step=$1
	local folder_name=$2
	local new_folder_name="$folder_name""_$perturb_step"
	local dest="$folder_name/$new_folder_name"
	mkdir -p "$dest"
	find ./"$folder_name" -maxdepth 1 -type f -not -name "*.castep" -not -name "*.txt" -not -name "*.csv" -print0 | xargs -0 -I {} cp {} "$dest"
	local param_file
	param_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.param")
	# setup param after perturbation
	param_after_perturb "$param_file"
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	# setup cell after perturbation
	cell_after_perturb "$cell_file" "$perturb_step" "$PERTURB_INCREMENT"
	# return new folder name
	setup_next_folder=$dest
}

function start_job {
	local current_dir
	current_dir=$(pwd)
	local job_dir=$1
	local job_type=$2
	local log_path=$3
	local result_path=$4
	local job_name
	job_name=$(find ./"$job_dir" -maxdepth 1 -type f -name "*.cell" | awk '{filename=$NF; sub(/\.[^.]+$/, "", filename); print filename}')
	local castep_command
	local castep_file="$job_name.castep"
	# Early exit if the job has been done.
	if [[ -f "$castep_file" && "$(grep -c "Finalisation time" "$castep_file")" -gt 0 ]]; then
		echo "Current castep job has been completed! Skip now"
		write_data "$castep_file" "$job_name" "$job_dir" "$job_type" "$result_path"
		return 1
	else
		cd "$job_dir" || exit
		case $job_type in
		U | u) castep_command="$castep_command_u" ;;
		alpha | Alpha) castep_command="$castep_command_alpha" ;;
		*) exit ;;
		esac
		# Here is the command to start calculation
		# Use a single & to move the job to background
		# standalone when command needs jobname
		# $castep_command "$job_name" 2>&1 | tee -a "$current_dir"/log_"$job_type".txt
		# cluster, only script needed
		$castep_command 2>&1 | tee -a "$current_dir"/log_"$job_type".txt
		cd "$current_dir" || exit
		monitor_job_done "$job_dir" "$job_type" "$result_path"
	fi
}

function monitor_job_done {
	local dest=$1
	local job_type=$2
	local local_result_path=$3
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
	write_data "$finished_castep_file" "$finished_job_name" "$dest" "$job_type" "$local_result_path"
}

function write_data {
	local castep_file=$1
	local finished_job_name=$2
	local dest=$3
	local cell_file
	cell_file=$(find ./"$dest" -maxdepth 1 -type f -name "*.cell")
	local job_type="$4"
	local local_result_path=$5
	local instant_result
	instant_result=result_"$job_type".csv
	local number_of_species
	number_of_species=$(awk '/%BLOCK HUBBARD_U/,/%ENDBLOCK HUBBARD_U/ {if (NF>2) print}' "$cell_file" | wc -l)
	for i in $(seq 1 "$number_of_species"); do
		local results_1
		results_1=$(grep -Ei "[[:blank:]]+$i[[:blank:]]+1 Total" "$castep_file" | awk 'NR==1 {printf "%.16f, ", $4}; NR==2 {printf "%.16f, ", $4}; END {printf "%.16f", $4} ORS=""')
		printf "%s, %i, %s\n" "$finished_job_name" "$i" "$results_1" >>"$local_result_path"
		printf "%s, %i, %s\n" "$finished_job_name" "$i" "$results_1" >>"$instant_result"
		local results_2
		results_2=$(grep -Ei "[[:blank:]]+$i[[:blank:]]+2 Total" "$castep_file" | awk 'NR==1 {printf "%.16f, ", $4}; NR==2 {printf "%.16f, ", $4}; END {printf "%.16f", $4} ORS=""')
		printf "%s, %i, %s\n" "$finished_job_name" "$i" "$results_2" >>"$local_result_path"
		printf "%s, %i, %s\n" "$finished_job_name" "$i" "$results_2" >>"$instant_result"
	done
}

function read_data {
	local i=$1
	local job_type=$2
	local folder_name=U_"$i"_"$job_type"
	cat "$folder_name"/result_"$job_type".csv >>result_"$job_type"_final.csv
}

# after cd into SEED_PATH
function routine {
	local init_u=$1
	local i=$2
	local init_elec_energy_tol=$3
	local job_type=$4
	local log_path=$5
	local PERTURB_TIMES=$6
	setup_before_perturb "$init_u" "$i" "$init_elec_energy_tol" "$job_type"
	init_folder="$setup_init_folder"
	local local_result_path="$init_folder"/result_"$job_type".csv
	: >"$local_result_path"
	# run castep
	# castep $SEED_PATH/$SEED_NAME
	# monitor result
	start_job "$init_folder" "$job_type" "$log_path" "$local_result_path"
	# echo  "Setup next perturbation step\r"
	for j in $(seq 1 "$PERTURB_TIMES"); do
		setup_after_perturb "$j" "$init_folder"
		next_folder=$setup_next_folder
		start_job "$next_folder" "$job_type" "$log_path" "$local_result_path"
	done
}

function create_log {
	local job_type=$1
	local current_dir
	current_dir=$(pwd)
	local log_path
	log_path="$current_dir"/"$SEED_PATH"/log_"$job_type".txt
	true >"$log_path"
	echo log_path
}

function setup {
	init_u=$1
	init_elec_energy_tol=$2
	step=$3
	final_U=$4
	job_type_input "$5"
	job_type=$input_job_type
	log_path=$(create_log "$job_type")
	printf "Jobname, Channel ID, Before SCF, 1st SCF, Last SCF\n" >"$SEED_PATH"/result_"$job_type".csv
}

function setup_perturbation {
	PERTURB_TIMES=$1
	PERTURB_INCREMENT=$2
	if [[ $PERTURB_INCREMENT == '' ]]; then
		PERTURB_INCREMENT=0.05
	fi
}

function setup_castep_command {
	castep_command_u=$1
	castep_command_alpha=$2
}

function serial {
	cd "$SEED_PATH" || exit
	for i in $(seq 0 "$step" "$final_U"); do
		routine "$init_u" "$i" "$init_elec_energy_tol" "$job_type" "$log_path" "$PERTURB_TIMES"
	done
	for i in $(seq 0 "$step" "$final_U"); do
		read_data "$i" "$job_type"
	done
	echo "Result:"
	cat result_"$job_type".csv
}

function parallel {
	local N=$1
	cd "$SEED_PATH" || exit
	for i in $(seq 0 "$step" "$final_U"); do
		(
			# .. do your stuff here
			routine "$init_u" "$i" "$init_elec_energy_tol" "$job_type" "$log_path" "$PERTURB_TIMES"
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
	for i in $(seq 0 "$step" "$final_U"); do
		read_data "$i" "$job_type"
	done
	echo "Result:"
	cat result_"$job_type".csv
	echo "all done"
}

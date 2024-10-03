use std::fmt::Display;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::seed_settings::JobType;

use self::program_mode::ProgramMode;

pub mod program_mode;

#[derive(Parser)]
#[command(author, version,about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: JobCommands,
}

#[derive(Subcommand)]
pub enum JobCommands {
    /// Read from existing result folder
    Read(ReadArgs),
    /// Start calculations
    Calc(CalcArgs),
}

#[derive(Debug, Args, Clone)]
pub struct ReadArgs {
    pub(crate) result_path: String,
    #[arg(short, long)]
    pub(crate) jobtype: Option<JobType>,
    #[arg(long)]
    pub(crate) init_input_u: Option<f64>,
    #[arg(long)]
    pub(crate) step_u: Option<f64>,
    #[arg(long)]
    pub(crate) final_u: Option<f64>,
    #[arg(long)]
    pub(crate) perturb_init: Option<f64>,
    #[arg(long)]
    pub(crate) perturb_step: Option<f64>,
    #[arg(long)]
    pub(crate) perturb_final: Option<f64>,
    #[arg(long)]
    pub(crate) perturb_times: Option<i64>,
}

#[derive(Debug)]
pub struct ReadArgsError;

impl Display for ReadArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error in reading parameters from result folder name")
    }
}

impl std::error::Error for ReadArgsError {}

impl ReadArgs {
    pub fn set_from_folder_name(&mut self) -> Result<(), ReadArgsError> {
        let result_path = Path::new(&self.result_path);
        let stem = result_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let mut components = stem.split('_').rev();
        self.perturb_times = components.next().and_then(|s| s.parse::<i64>().ok());
        components.next();
        self.perturb_final = components.next().and_then(|s| s.parse::<f64>().ok());
        self.perturb_step = components.next().and_then(|s| s.parse::<f64>().ok());
        self.perturb_init = components.next().and_then(|s| s.parse::<f64>().ok());
        self.final_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.step_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.init_input_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.jobtype = components.next().and_then(|s| JobType::from_str(s).ok());

        if self.jobtype.is_some()
            && self.init_input_u.is_some()
            && self.step_u.is_some()
            && self.final_u.is_some()
            && self.perturb_init.is_some()
            && self.perturb_step.is_some()
            && self.perturb_final.is_some()
            && self.perturb_times.is_some()
        {
            Ok(())
        } else {
            Err(ReadArgsError)
        }
    }
    pub fn invoke(&self) {
        Command::new("./auto_hubbard_linux.sh")
            .arg(&self.result_path)
            .arg(self.jobtype.unwrap().to_string())
            .arg("read")
            .arg(format!("{}", &self.init_input_u.unwrap()))
            .arg(format!("{}", &self.step_u.unwrap()))
            .arg(format!("{}", &self.final_u.unwrap()))
            .arg(format!("{}", &self.perturb_times.unwrap()))
            .output()
            .expect("Failed to start auto_hubbard_linux.sh in read mode");
    }
}

#[derive(Args)]
pub struct CalcArgs {
    #[arg(short, long, default_value_t = ProgramMode::Parallel)]
    pub(crate) mode: ProgramMode,
    #[arg(long = "seedpath")]
    pub(crate) seed_path: String,
    #[arg(short, long)]
    pub(crate) jobtype: JobType,
    #[arg(long, default_value_t = 0.0)]
    pub(crate) init_input_u: f64,
    #[arg(long, default_value_t = 2.0)]
    pub(crate) step_u: f64,
    #[arg(long, default_value_t = 12.0)]
    pub(crate) final_u: f64,
    #[arg(long, default_value_t = 0.05)]
    pub(crate) perturb_init: f64,
    #[arg(long, default_value_t = 0.05)]
    pub(crate) perturb_step: f64,
    #[arg(long, default_value_t = 0.25)]
    pub(crate) perturb_final: f64,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

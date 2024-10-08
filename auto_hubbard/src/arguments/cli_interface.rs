#![allow(dead_code)]
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::seed_settings::JobType;

use super::program_mode::ProgramMode;

#[derive(Parser)]
#[command(author, version,about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: JobCommands,
}

impl Cli {
    pub fn command_mut(&mut self) -> &mut JobCommands {
        &mut self.command
    }
}

#[derive(Subcommand)]
#[command(version, about)]
pub enum JobCommands {
    /// Read from existing result folder
    Read(ReadArgs),
    /// Start calculations
    Calc(CalcArgs),
}

#[derive(Debug, Args, Clone, Default)]
#[command(version, about)]
pub struct ReadArgs {
    /// Path to result folder, e.g.: XXX_[jobtype]_[init_input_u]_[step_u]_[final_u]_[perturb_init]_[perturb_step]_[perturb_final]_STEPS_[perturb_times]
    pub(crate) result_path: String,
    #[arg(short, long)]
    /// Interpreted from the folder name by default.
    pub(crate) jobtype: Option<JobType>,
    #[arg(long)]
    /// Interpreted from the folder name by default.
    pub(crate) init_input_u: Option<f64>,
    #[arg(long)]
    /// Interpreted from the folder name by default.
    pub(crate) step_u: Option<f64>,
    #[arg(long)]
    /// Interpreted from the folder name by default.
    pub(crate) final_u: Option<f64>,
    #[arg(long)]
    /// Interpreted from the folder name by default.
    /// Decide how many rounds of perturbations to be read
    pub(crate) perturb_times: Option<i64>,
}

impl Display for ReadArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for ReadArgs {
    type Err = ReadArgsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ReadArgs::new_with_folder_name(s)
    }
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
    pub fn new_with_folder_name(folder_name: &str) -> Result<Self, ReadArgsError> {
        let mut new_args = Self {
            result_path: folder_name.to_string(),
            ..Self::default()
        };
        match new_args.set_from_folder_name() {
            Ok(_) => Ok(new_args),
            Err(e) => Err(e),
        }
    }
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
        self.final_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.step_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.init_input_u = components.next().and_then(|s| s.parse::<f64>().ok());
        self.jobtype = components.next().and_then(|s| JobType::from_str(s).ok());

        if self.jobtype.is_some()
            && self.init_input_u.is_some()
            && self.step_u.is_some()
            && self.final_u.is_some()
            && self.perturb_times.is_some()
        {
            Ok(())
        } else {
            Err(ReadArgsError)
        }
    }
    pub fn invoke(&self) -> Result<(), io::Error> {
        let output = Command::new("./auto_hubbard_linux.sh")
            .arg(&self.result_path)
            .arg(self.jobtype.unwrap().to_string())
            .arg("read")
            .arg(format!("{}", &self.init_input_u.unwrap()))
            .arg(format!("{}", &self.step_u.unwrap()))
            .arg(format!("{}", &self.final_u.unwrap()))
            .arg(format!("{}", &self.perturb_times.unwrap()))
            .output()
            .expect("Failed to start auto_hubbard_linux.sh in read mode");
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)
    }
}

#[derive(Args)]
#[command(version, about)]
pub struct CalcArgs {
    /// Path to the seed folder including `.cell`, `.param` and other necessary files.
    pub(crate) seed_path: String,
    /// `u` or `alpha`
    pub(crate) jobtype: JobType,
    /// `parallel` or `serial`
    #[arg(short, long, default_value_t = ProgramMode::Parallel)]
    pub(crate) mode: ProgramMode,
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

impl CalcArgs {
    pub fn invoke(&self) -> Result<(), io::Error> {
        let output = Command::new("./auto_hubbard_linux.sh")
            .arg(&self.seed_path)
            .arg(self.jobtype.to_string())
            .arg(self.mode.to_string())
            .arg(format!("{}", &self.init_input_u))
            .arg(format!("{}", &self.step_u))
            .arg(format!("{}", &self.final_u))
            .arg(format!("{}", &self.perturb_init))
            .arg(format!("{}", &self.perturb_step))
            .arg(format!("{}", &self.perturb_final))
            .output()
            .expect("Failed to start auto_hubbard_linux.sh in read mode");
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

use clap::ValueEnum;
use std::path::PathBuf;
use thiserror::Error;

pub use clap::Parser;

#[derive(Debug, clap::Parser)]
/// Sorting data of `result_u_final.csv` and `result_alpha_final.csv`
#[command(name = "hubbard_data")]
#[command(about = "Data extraction from Hubbard U/Alpha perturbation runs and plotting.")]
pub struct HubbardDataCli {
    #[arg(short = 's')]
    result_folder: PathBuf,
    #[arg(short, long)]
    u_perturb_val: f64,
    #[arg(short, long)]
    alpha_perturb_val: f64,
    #[arg(short, long)]
    verbose: Option<bool>,
    #[arg(short, long)]
    mode: Option<Mode>,
}

impl HubbardDataCli {
    pub fn result_folder(&self) -> &PathBuf {
        &self.result_folder
    }

    pub fn u_perturb_val(&self) -> f64 {
        self.u_perturb_val
    }

    pub fn alpha_perturb_val(&self) -> f64 {
        self.alpha_perturb_val
    }

    pub fn verbose(&self) -> Option<bool> {
        self.verbose
    }

    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// Return the perturb value(s) in `PerturbValue` enum
    /// based on `Mode`
    pub fn perturb_value(&self) -> PerturbValue {
        match self.mode.unwrap_or_default() {
            Mode::Both => PerturbValue::Both((self.u_perturb_val(), self.alpha_perturb_val())),
            Mode::U => PerturbValue::Single(self.u_perturb_val()),
            Mode::Alpha => PerturbValue::Single(self.alpha_perturb_val()),
        }
    }
}

/// An enum to hold two different return types of `HubbardDataCli::perturb_value(&self)`
#[derive(Debug, Clone, Copy)]
pub enum PerturbValue {
    /// (u_perturb_val, alpha_perturb_val)
    Both((f64, f64)),
    /// (u or alpha perturb_value)
    Single(f64),
}

#[derive(Debug, Error)]
pub enum PerturbValueConversionError {
    #[error("The mode is not `Single`")]
    NotSingle,
    #[error("The mode is not `Both`")]
    NotBoth,
}

impl PerturbValue {
    pub fn try_into_single(self) -> Result<f64, PerturbValueConversionError> {
        if let Self::Single(v) = self {
            Ok(v)
        } else {
            Err(PerturbValueConversionError::NotSingle)
        }
    }

    pub fn try_into_both(self) -> Result<(f64, f64), PerturbValueConversionError> {
        if let Self::Both(v) = self {
            Ok(v)
        } else {
            Err(PerturbValueConversionError::NotBoth)
        }
    }
}

/// An enum to define the job mode of the program
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum Mode {
    #[default]
    /// Analyze both U and Alpha job results.
    Both,
    /// Analyze result only from U job
    U,
    /// Analyze result only from Alpha job
    Alpha,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn it_works() {
        HubbardDataCli::command().debug_assert();
    }
}

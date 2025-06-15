use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
/// Sorting data of `result_u_final.csv` and `result_alpha_final.csv`
#[command(name = "hubbard_data")]
#[command(about = "Data extraction from Hubbard U/Alpha perturbation runs and plotting.")]
pub struct HubbardDataCli {
    #[arg(short = 's')]
    result_folder: Option<PathBuf>,
    #[arg(short, long)]
    u_perturb_val: f64,
    #[arg(short, long)]
    alpha_perturb_val: f64,
    #[arg(short, long)]
    verbose: Option<bool>,
}

impl HubbardDataCli {
    pub fn result_folder(&self) -> Option<&PathBuf> {
        self.result_folder.as_ref()
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
}

pub use clap::Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn it_works() {
        HubbardDataCli::command().debug_assert();
    }
}

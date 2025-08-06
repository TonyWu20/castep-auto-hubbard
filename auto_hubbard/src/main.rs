use std::io;

use crate::arguments::ReadArgs;
use arguments::Cli;
use clap::Parser;
use inquire::CustomType;

mod arguments;
mod errors;
mod seed_settings;
mod pipeline {
    #![allow(dead_code)]
    //! Things to do:
    //! 1. Receive a folder path, check existence of `.cell` and `.param`
    //! 2. Deserialization
    //!     1. Deserialize `.cell` with `castep_cell_data::from_str::<CellFile>`
    //!     2. Deserialize `.param` with `castep_cell_data::from_str::<ParamFile>`
    //! 3. Create `HubbardUCell<Init>` with `HubbardUCell::<Init>::from_cell_file(cell_file: CellFile)`, create
    //!    `HubbardUParam<Init>` with `HubbardUParam::from_param(param_file:ParamFile)`

    use std::sync::Arc;

    use castep_cell_data::param::electronic_minimisation::ElecEnergyTol;
    use serde::{Deserialize, Serialize};

    use crate::seed_settings::JobType;
    mod sequence;

    pub use sequence::Sequence;

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    pub struct HubArguments {
        /// A value very small and close to zero, to trick `CASTEP` into LDA+U even if
        /// U is meant to be zero
        /// Default: 1e-7
        init_hubbard_u: f64,
        /// The `elec_energy_tol` for the first run without perturbation
        /// Default: ElecEnergyTol {value: 1e-5, unit:None}
        init_elec_energy_tol: ElecEnergyTol,
        /// Beginning of the `U` series
        u_start: f64,
        /// Increment of `U`
        u_step: f64,
        /// Ending of the `U` series
        u_end: f64,
        /// Beginning of the `alpha` series
        alpha_start: f64,
        /// Increment of `alpha`
        alpha_step: f64,
        /// Ending of the `alpha` series
        alpha_end: f64,
        /// Determine `U` or `Alpha` run
        job_type: JobType,
    }
    impl HubArguments {
        fn u_values(&self) -> Arc<[f64]> {
            Sequence::new(self.u_start, self.u_step, self.u_end)
                .map(|v| v + self.init_hubbard_u)
                .collect::<Arc<[f64]>>()
        }
        fn alpha_values(&self) -> Arc<[f64]> {
            Sequence::new(self.alpha_start, self.alpha_step, self.alpha_end)
                .map(|v| v + self.init_hubbard_u)
                .collect::<Arc<[f64]>>()
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct HubUValues {
        pub u: f64,
        pub alpha: f64,
    }

    impl HubUValues {
        pub fn new(u: f64, alpha: f64) -> Self {
            Self { u, alpha }
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum HubUSetup {
        U(f64, f64),
        Alpha(f64, f64),
    }

    impl HubUSetup {
        pub fn init(init_hubbard_u: f64, job_type: JobType) -> Self {
            match job_type {
                JobType::U => HubUSetup::U(init_hubbard_u, init_hubbard_u),
                JobType::Alpha => HubUSetup::Alpha(init_hubbard_u, init_hubbard_u),
            }
        }
        /// Applied a function to a single field of the tuple variants
        /// For `U`: apply `f` on the first field
        pub fn map_u<F>(&self, f: F) -> Self
        where
            F: FnOnce(&f64) -> f64,
        {
            match self {
                HubUSetup::U(u, a) => Self::U(f(u), *a),
                HubUSetup::Alpha(u, a) => Self::Alpha(f(u), *a),
            }
        }
        /// Applied a function to a single field of the tuple variants
        /// For `Alpha`: apply `f` on the second field
        pub fn map_alpha<F>(&self, f: F) -> Self
        where
            F: FnOnce(&f64) -> f64,
        {
            match self {
                HubUSetup::U(u, a) => Self::U(*u, f(a)),
                HubUSetup::Alpha(u, a) => Self::Alpha(*u, f(a)),
            }
        }
        /// Applied a function to both fields of the tuple variants
        pub fn map<F>(&self, f: F) -> Self
        where
            F: Fn(&f64) -> f64,
        {
            match self {
                HubUSetup::U(u, a) => Self::U(f(u), f(a)),
                HubUSetup::Alpha(u, a) => Self::Alpha(f(u), f(a)),
            }
        }
        pub fn set_u(&self, current_u: f64) -> Self {
            self.map_u(|curr| curr + current_u)
        }
        pub fn alpha_perturb(&self, new_alpha: f64) -> Self {
            self.map_alpha(|curr_alpha| curr_alpha + new_alpha)
        }
        pub fn u_value(&self) -> f64 {
            match self {
                HubUSetup::U(u, _) => *u,
                HubUSetup::Alpha(u, a) => *a,
            }
        }
        pub fn alpha_value(&self) -> f64 {
            match self {
                HubUSetup::U(_, a) => *a,
                HubUSetup::Alpha(a, _) => *a,
            }
        }
    }

    impl From<HubUSetup> for (f64, f64) {
        fn from(value: HubUSetup) -> Self {
            (value.u_value(), value.alpha_value())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use crate::{
            pipeline::{HubUSetup, HubUValues},
            seed_settings::JobType,
        };

        use super::Sequence;

        #[test]
        fn test_seq() {
            let initial_u = 1e-7;
            let u_range = Sequence::new(-10.0, 1.0, 10.0);
            dbg!(u_range
                .map(|f| format!("{f:.2}"))
                .collect::<Arc<[String]>>());
            let alpha_range = Sequence::new(0.05, 0.05, 0.25);
            dbg!(
                &alpha_range.collect::<Arc<[f64]>>().len(),
                alpha_range
                    .map(|f| format!("{f:.2}"))
                    .collect::<Arc<[String]>>()
            );
            let mock_run = u_range
                .map(|u| HubUSetup::init(initial_u, JobType::U).set_u(u))
                .map(|hub_u_value| {
                    (
                        hub_u_value,
                        alpha_range
                            .map(move |alpha| hub_u_value.alpha_perturb(alpha))
                            .collect::<Arc<[HubUSetup]>>(),
                    )
                })
                .map(|(u_a, a_perturbed)| {
                    (
                        u_a.map(|f| {
                            format!("{f:.16}")
                                .parse::<f64>()
                                .expect("truncated_u should still be `f64`")
                        }),
                        a_perturbed,
                    )
                })
                .collect::<Arc<[(HubUSetup, Arc<[HubUSetup]>)]>>();
            dbg!(mock_run);
            let neg_range = Sequence::new(5, -1, 0);
            dbg!(neg_range.collect::<Arc<[i32]>>());
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut cli = Cli::parse();
    match &mut cli.command_mut() {
        arguments::JobCommands::Read(args) => {
            let set = args.set_from_folder_name();
            if let Err(e) = set {
                println!("{e}");
                let new_args = CustomType::<ReadArgs>::new("Please enter the name of the result folder (e.g.: XXX_[jobtype]_[init_input_u]_[step_u]_[final_u]_[perturb_init]_[perturb_step]_[perturb_final]_STEPS_[perturb_times])")
                    .prompt().unwrap();
                new_args.invoke()?;
            }
            args.invoke()
        }
        arguments::JobCommands::Calc(calc_args) => calc_args.invoke(),
    }
}

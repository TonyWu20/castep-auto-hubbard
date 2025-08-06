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

    use std::{
        ops::{Add, AddAssign, Div, Neg, Sub},
        sync::Arc,
    };

    use castep_cell_data::param::electronic_minimisation::ElecEnergyTol;
    use serde::{Deserialize, Serialize};

    use crate::seed_settings::JobType;

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
            let u_sequence = Sequence::new(self.u_start, self.u_step, self.u_end);
            u_sequence
                .map(|v| v + self.init_hubbard_u)
                .collect::<Arc<[f64]>>()
        }
    }

    pub trait NumLike:
        Add<Output = Self>
        + AddAssign
        + PartialEq
        + PartialOrd
        + Copy
        + Sub<Output = Self>
        + Neg<Output = Self>
        + Div<Output = Self>
    {
    }
    impl<
            T: Add<Output = Self>
                + AddAssign
                + PartialEq
                + PartialOrd
                + Copy
                + Sub<Output = Self>
                + Neg<Output = Self>
                + Div<Output = Self>,
        > NumLike for T
    {
    }

    /// To represent and generate a sequence of item
    /// It is inclusive: $[`start`, `end`]$ with interval of `step`
    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    pub struct Sequence<T>
    where
        T: NumLike,
    {
        start: T,
        step: T,
        end: T,
        current: Option<T>,
    }

    impl<T> Sequence<T>
    where
        T: NumLike,
    {
        pub fn new(start: T, step: T, end: T) -> Self {
            Self {
                start,
                step,
                end,
                current: None,
            }
        }
    }

    impl<T: NumLike> Iterator for Sequence<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.current = self.current.map_or(Some(self.start), |curr| {
                let next = curr + self.step;
                if self.start <= self.end {
                    (next <= self.end).then_some(next)
                } else {
                    (next >= self.end).then_some(next)
                }
            });
            self.current
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use super::Sequence;

        #[test]
        fn test_seq() {
            let u_range = Sequence::new(-10.0, 0.1, 10.0);
            dbg!(u_range
                .map(|f| format!("{f:.2}"))
                .collect::<Arc<[String]>>());
            let alpha_range = Sequence::new(0.05, 0.04, 0.25);
            dbg!(
                &alpha_range.collect::<Arc<[f64]>>().len(),
                alpha_range
                    .map(|f| format!("{f:.2}"))
                    .collect::<Arc<[String]>>()
            );
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

use std::path::{Path, PathBuf};

use polars::prelude::{DataType, Expr, col, fold_exprs, lit};

use crate::analysis::{Pipeline, csv_path::CSVPath};

/// Represents the job type of hubbard perturbation run:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U;

/// Represents the job type of hubbard perturbation run:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alpha;

/// Common methods for a type representing a Hubbard perturbation task.
pub trait JobType: Sized {
    /// Current job type in `String`
    fn job_type() -> String;
    /// result csv filename
    fn csv_path<P: AsRef<Path>>(directory: P) -> Pipeline<Self, CSVPath<Self>, PathBuf>;
    /// Alias for column of perturbation
    fn nth_perturb_col_alias() -> String;
    /// Alias for column of slope for 1st SCF - Before SCF.
    fn slope_first_col_alias() -> String;
    /// Alias for column of slope for Final SCF - Before SCF.
    fn slope_final_col_alias() -> String;
    /// Alias for column of slope for `slope_first` - `slope_final`
    fn delta_slope_col_alias() -> String;
    /// Generate a column marking the perturbation step from column "Jobname" of the csv
    fn perturb_expr() -> Expr;
    /// Calculate the slope from data
    fn slope_expr(perturb_step_val: f64) -> [Expr; 2] {
        let matched_perturb_step_col = Self::nth_perturb_col_alias();
        let calc_expr = |perturb_step_col: Expr, scf_col: Expr, alias: &str| {
            fold_exprs(
                lit(1),
                |acc, val| (acc * val).map(Some),
                // perturb_step * perturb_step_val * (1 / ΔSCF)
                [perturb_step_col * lit(perturb_step_val), lit(1.0) / scf_col],
                false,
                Some(DataType::Float64),
            )
            .alias(alias)
        };
        [
            calc_expr(
                col(&matched_perturb_step_col),
                col("S1-S0"),
                &Self::slope_first_col_alias(),
            ),
            calc_expr(
                col(&matched_perturb_step_col),
                col("SF-S0"),
                &Self::slope_final_col_alias(),
            ),
        ]
    }
}

impl JobType for U {
    fn csv_path<P: AsRef<Path>>(directory: P) -> Pipeline<Self, CSVPath<Self>, PathBuf> {
        Pipeline::new(directory.as_ref().join("result_u_final.csv"))
    }

    /// "u_pert"
    fn nth_perturb_col_alias() -> String {
        "u_pert".into()
    }

    /// "u/S1-S0"
    fn slope_first_col_alias() -> String {
        "u/S1-S0".into()
    }

    /// "u/SF-S0"
    fn slope_final_col_alias() -> String {
        "u/SF-S0".into()
    }

    /// "n1-nF_U"
    fn delta_slope_col_alias() -> String {
        "n1-nF_U".into()
    }

    fn perturb_expr() -> Expr {
        col("Jobname")
            .str()
            .extract(
                lit(
                    // Format: U_0_u_0 or U_0_alpha_0
                    r"u_(\d+)",
                ),
                1,
            )
            .cast(DataType::Int32)
            .alias(Self::nth_perturb_col_alias())
    }

    /// "U"
    fn job_type() -> String {
        "U".into()
    }
}

impl JobType for Alpha {
    fn csv_path<P: AsRef<Path>>(directory: P) -> Pipeline<Self, CSVPath<Self>, PathBuf> {
        Pipeline::new(directory.as_ref().join("result_alpha_final.csv"))
    }

    /// "alpha_pert"
    fn nth_perturb_col_alias() -> String {
        "alpha_pert".into()
    }

    /// "alpha/S1-S0"
    fn slope_first_col_alias() -> String {
        "alpha/S1-S0".into()
    }

    /// "alpha/SF-S0"
    fn slope_final_col_alias() -> String {
        "alpha/SF-S0".into()
    }

    /// "n1-nF_Alpha"
    fn delta_slope_col_alias() -> String {
        "n1-nF_Alpha".into()
    }

    fn perturb_expr() -> Expr {
        col("Jobname")
            .str()
            .extract(
                lit(
                    // Format: U_0_u_0 or U_0_alpha_0
                    r"alpha_(\d+)",
                ),
                1,
            )
            .cast(DataType::Int32)
            .alias(Self::nth_perturb_col_alias())
    }

    /// "Alpha"
    fn job_type() -> String {
        "Alpha".into()
    }
}

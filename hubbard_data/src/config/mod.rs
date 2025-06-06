use polars::prelude::{DataType, Expr, col, fold_exprs, lit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobType {
    U,
    Alpha,
}

impl JobType {
    /// result csv filename
    /// # Return
    /// - `JobType::U` => "result_u_final.csv"
    /// - `JobType::Alpha` => "result_alpha_final.csv"
    pub fn filename(&self) -> String {
        match self {
            JobType::U => "result_u_final.csv".to_string(),
            JobType::Alpha => "result_alpha_final.csv".to_string(),
        }
    }

    /// result of channels filename
    /// # Return
    /// - "channel_{id}_sorted_{JobType}.csv"
    pub fn channel_filename(&self, channel_id: u32) -> String {
        format!(
            "channel_{}_sorted_{}.csv",
            channel_id,
            match self {
                JobType::U => "U",
                JobType::Alpha => "alpha",
            }
        )
    }
    /// Alias for column of perturbation
    /// # Return
    /// - `JobType::U` => "u_pert",
    /// - `JobType::Alpha` => "alpha_pert"
    pub fn perturb_col_alias(&self) -> &str {
        match self {
            JobType::U => "u_pert",
            JobType::Alpha => "alpha_pert",
        }
    }

    /// Alias for column of slope for 1st SCF - Before SCF.
    /// # Return
    /// - `JobType::U` => "u/S1-S0",
    /// - `JobType::Alpha` => "alpha/S1-S0"
    pub fn slope_first_col_alias(&self) -> &str {
        match self {
            JobType::U => "u/S1-S0",
            JobType::Alpha => "alpha/S1-S0",
        }
    }

    /// Alias for column of slope for Final SCF - Before SCF.
    /// # Return
    /// - `JobType::U` => "u/SF-S0",
    /// - `JobType::Alpha` => "alpha/SF-S0"
    pub fn slope_final_col_alias(&self) -> &str {
        match self {
            JobType::U => "u/SF-S0",
            JobType::Alpha => "alpha/SF-S0",
        }
    }

    /// Generate a column marking the perturbation step from column "Jobname" of the csv
    /// # Return
    /// - `Expr`: A column of Int32 with column name "u_pert" or "alpha_pert"
    pub fn perturb_expr(&self) -> Expr {
        col("Jobname")
            .str()
            .extract(
                lit(match self {
                    // Format: U_0_u_0 or U_0_alpha_0
                    JobType::U => r"u_(\d+)",
                    JobType::Alpha => r"alpha_(\d+)",
                }),
                1,
            )
            .cast(DataType::Int32)
            .alias(self.perturb_col_alias())
    }
    /// Calculate the slope from data
    /// # Return
    /// - `[col(u|alpha/s1-s0), col(u|alpha/sf-s0)]`
    pub fn slope_expr(&self, perturb_step_val: f64) -> [Expr; 2] {
        let matched_perturb_step_col = self.perturb_col_alias();
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
                col(matched_perturb_step_col),
                col("S1-S0"),
                self.slope_first_col_alias(),
            ),
            calc_expr(
                col(matched_perturb_step_col),
                col("SF-S0"),
                self.slope_final_col_alias(),
            ),
        ]
    }
}

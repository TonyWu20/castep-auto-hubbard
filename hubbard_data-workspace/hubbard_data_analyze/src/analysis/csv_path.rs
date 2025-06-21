use crate::{JobType, TotalView};
use std::{marker::PhantomData, path::PathBuf};

use polars::{
    error::PolarsError,
    lazy::dsl::sum_horizontal,
    prelude::{DataType, LazyCsvReader, LazyFileListReader, LazyFrame, col, lit},
};

use super::{Pipeline, ViewType};

#[derive(Debug, Clone, Copy)]
/// A type holding the path to the target csv file as well as the generic type signature `JobType`
pub struct CSVPath<T: JobType>(PhantomData<T>);

impl<T: JobType> ViewType<T> for CSVPath<T> {}

impl<T: JobType> Pipeline<T, CSVPath<T>, PathBuf> {
    /// Process the data according to our algorithm.
    /// We want to sum the results of two spins:
    /// ```csv
    /// Jobname       |Channel ID|Spin
    /// ./U_0_u/ZnO_LR|1         |1
    /// ./U_0_u/ZnO_LR|1         |2
    /// ```
    /// And calculate the perturbation response at different perturbation values:
    /// `n1-nF = perturb_val * perturb_times / (s1-s0) - perturb_val * perturb_times / (sf-s0)`
    /// The processed total view will have the followig columns:
    /// [ "Channel ID", "U", "S1-S0", "SF-S0", "u/S1-S0 | alpha/s1-S0", "u/SF-S0 | alpha/SF-S0", "u_pert | alpha_pert", "n1-nF"]
    pub fn process_data(
        &self,
        perturb_val: f64,
    ) -> Result<Pipeline<T, TotalView<T>, LazyFrame>, PolarsError> {
        let [jobname, channel_id, scf_0, scf_1, scf_last] =
            ["Jobname", "Channel ID", "Before SCF", "1st SCF", "Last SCF"];
        let dataframe = LazyCsvReader::new(&self.data)
            .with_has_header(true)
            .finish()?
            .select([
                col(jobname),
                col(channel_id).cast(DataType::UInt32),
                col(scf_0).cast(DataType::Float64),
                col(scf_1).cast(DataType::Float64),
                col(scf_last).cast(DataType::Float64),
            ])
            .filter(
                // Remove non-perturbed entries, e.g.: ./U_0_u/ZnO_LR
                col(jobname).str().count_matches(lit("/"), true).eq(lit(3)),
            )
            // We want to sum the results of two spins:
            // Jobname       |Channel ID|Spin
            // ./U_0_u/ZnO_LR|1         |1
            // ./U_0_u/ZnO_LR|1         |2
            // So use `group_by_stable` (preserve order)
            .group_by_stable([col(jobname), col(channel_id)])
            // Aggregate the `LazyGroupBy`
            .agg([
                col("Before SCF").sum().alias("S0"),
                col("1st SCF").sum().alias("S1"),
                col("Last SCF").sum().alias("SF"),
            ])
            .select([
                col(jobname),
                col(channel_id),
                // Extract U increment value from jobname
                col(jobname)
                    .clone()
                    .alias("U")
                    .str()
                    .extract(lit(r"U_(\d+)"), 1)
                    .cast(DataType::Int32),
                //
                T::perturb_expr(),
                sum_horizontal([col("S1"), col("S0") * lit(-1)], false)?.alias("S1-S0"),
                sum_horizontal([col("SF"), col("S0") * lit(-1)], false)?.alias("SF-S0"),
            ])
            // "u_pert"/"alpha_pert"
            .with_columns(T::slope_expr(perturb_val))
            .with_column(
                // perturb_val * perturb_times / (s1-s0) - perturb_val * perturb_times / (sf-s0)
                sum_horizontal(
                    [
                        col(T::slope_first_col_alias()),
                        col(T::slope_final_col_alias()) * lit(-1),
                    ],
                    false,
                )?
                .alias("n1-nF"),
            )
            // Drop "Jobname"
            .select([
                col(channel_id),
                col("U"),
                col("S1-S0"),
                col("SF-S0"),
                col(T::slope_first_col_alias()),
                col(T::slope_final_col_alias()),
                col(T::nth_perturb_col_alias()) * lit(perturb_val),
                col("n1-nF"),
            ]);
        Ok(Pipeline::new(dataframe))
    }
}

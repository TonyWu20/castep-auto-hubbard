use polars::prelude::{JoinType, LazyFrame};
use std::path::Path;

use polars::{
    error::PolarsError,
    frame::DataFrame,
    lazy::dsl::sum_horizontal,
    prelude::{DataType, IntoLazy, JoinArgs, LazyCsvReader, LazyFileListReader, col, lit},
};

use crate::config::JobType;

pub mod plot;

pub fn get_result(
    directory: &Path,
    perturb_val: f64,
    job_type: JobType,
) -> Result<DataFrame, anyhow::Error> {
    Ok(LazyCsvReader::new(directory.join(job_type.filename()))
        .with_has_header(true)
        .finish()?
        .select([
            col("Jobname"),
            col("Channel ID").cast(DataType::UInt32),
            col("Before SCF").cast(DataType::Float64),
            col("1st SCF").cast(DataType::Float64),
            col("Last SCF").cast(DataType::Float64),
        ])
        .filter(
            // Remove non-perturbed entries, e.g.: ./U_0_u/ZnO_LR
            col("Jobname")
                .str()
                .count_matches(lit("/"), true)
                .eq(lit(3)),
        )
        // We want to sum the results of two spins:
        // Jobname       |Channel ID|Spin
        // ./U_0_u/ZnO_LR|1         |1
        // ./U_0_u/ZnO_LR|1         |2
        // So use `group_by_stable` (preserve order)
        .group_by_stable([col("Jobname"), col("Channel ID")])
        // Aggregate the `LazyGroupBy`
        .agg([
            col("Before SCF").sum().alias("S0"),
            col("1st SCF").sum().alias("S1"),
            col("Last SCF").sum().alias("SF"),
        ])
        .select([
            col("Jobname"),
            col("Channel ID"),
            // Extract U increment value from jobname
            col("Jobname")
                .clone()
                .alias("U")
                .str()
                .extract(lit(r"U_(\d+)"), 1)
                .cast(DataType::Int32),
            //
            job_type.perturb_expr(),
            sum_horizontal([col("S1"), col("S0") * lit(-1)], false)?.alias("S1-S0"),
            sum_horizontal([col("SF"), col("S0") * lit(-1)], false)?.alias("SF-S0"),
        ])
        // "u_pert"/"alpha_pert"
        .with_columns(job_type.slope_expr(perturb_val))
        .with_column(
            // perturb_val * perturb_times / (s1-s0) - perturb_val * perturb_times / (sf-s0)
            sum_horizontal(
                [
                    col(job_type.slope_first_col_alias()),
                    col(job_type.slope_final_col_alias()) * lit(-1),
                ],
                false,
            )?
            .alias("n1-nF"),
        )
        // Drop "Jobname"
        .select([
            col("Channel ID"),
            col("U"),
            col("S1-S0"),
            col("SF-S0"),
            col(job_type.slope_first_col_alias()),
            col(job_type.slope_final_col_alias()),
            col(job_type.perturb_col_alias()) * lit(perturb_val),
            col("n1-nF"),
        ])
        .collect()?)
}

pub fn view_by_channel_id(
    df_u: &DataFrame,
    df_alpha: &DataFrame,
    channel_id: u32,
) -> Result<LazyFrame, PolarsError> {
    Ok(df_u
        .clone()
        .lazy()
        .filter(col("Channel ID").eq(lit(channel_id)))
        .select([col("U"), col("u_pert"), col("n1-nF").alias("n1-nF_U")])
        .join(
            df_alpha
                .clone()
                .lazy()
                .filter(col("Channel ID").eq(lit(channel_id)))
                .select([
                    col("U"),
                    col("alpha_pert"),
                    col("n1-nF").alias("n1-nF_Alpha"),
                ]),
            [col("U")],
            [col("U")],
            JoinArgs::new(JoinType::Left),
        ))
}

pub fn view_mean(channel_view: LazyFrame) -> Result<DataFrame, PolarsError> {
    channel_view
        .group_by_stable([col("U"), col("u_pert"), col("alpha_pert")])
        .agg([col("n1-nF_U").mean(), col("n1-nF_Alpha").mean()])
        .select([col("U"), col("n1-nF_U"), col("n1-nF_Alpha")])
        .collect()
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use polars::{
        io::SerWriter,
        prelude::{CsvWriter, col},
    };

    use crate::dataframe::{get_result, view_by_channel_id};

    #[test]
    fn test_df() {
        let result_folder = Path::new("../sorting");
        let mut result_df_u = get_result(result_folder, 0.05, crate::config::JobType::U).unwrap();
        let mut result_df_alpha =
            get_result(result_folder, 0.05, crate::config::JobType::Alpha).unwrap();
        println!("{}", result_df_u);
        let mut result_file = std::fs::File::create("test_sorting.csv").unwrap();
        CsvWriter::new(&mut result_file)
            .finish(&mut result_df_u)
            .unwrap();
        let ids = result_df_u
            .column("Channel ID")
            .unwrap()
            .unique_stable()
            .unwrap();
        println!("{:?}", ids.u32().unwrap());
        ids.u32().unwrap().iter().for_each(|i| {
            let channel_view_lz =
                view_by_channel_id(&result_df_u, &result_df_alpha, i.unwrap()).unwrap();
            let channel_view = channel_view_lz.clone().collect().unwrap();
            let channel_view_mean = channel_view_lz
                .group_by_stable([col("U"), col("u_pert"), col("alpha_pert")])
                .agg([col("n1-nF_U").mean(), col("n1-nF_Alpha").mean()])
                .select([col("U"), col("n1-nF_U"), col("n1-nF_Alpha")])
                .collect()
                .unwrap();
            println!("{}", channel_view);
            println!("{}", channel_view_mean);
        });
    }
}

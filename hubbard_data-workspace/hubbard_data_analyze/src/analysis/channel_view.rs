use polars::{
    error::PolarsError,
    frame::DataFrame,
    functions::concat_df_horizontal,
    prelude::{ChunkUnique, IntoLazy, LazyFrame, col, lit},
};

use crate::job_type::{Alpha, JobType, U};

use super::total_view::TotalView;

/// Select by channel id and only retrieve the final result columns
/// # Args:
/// - channel_id : existing channel id
impl<T: JobType> TotalView<T> {
    pub fn view_by_channel_id(&self, channel_id: u32) -> LazyFrame {
        self.dataframe()
            .clone()
            .lazy()
            .filter(col("Channel ID").eq(lit(channel_id)))
            .select([
                col("U"),
                col(T::nth_perturb_col_alias()),
                col("n1-nF").alias(T::delta_slope_col_alias()),
            ])
    }
}

/// The result of merging dataframes from `result_u_final` and `result_alpha_final`
pub struct MergedLazyFrame(pub(crate) LazyFrame);

impl std::ops::Deref for MergedLazyFrame {
    type Target = LazyFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MergedLazyFrame {
    /// Concat the U view and Alpha view
    /// Put the U view on the left as the main body, with columns ["U", "u_pert", "n1-nF_U"]
    /// and Alpha view on the right with columns ["alpha_pert", "n1-nF_Alpha"]
    pub fn merge_u_alpha_channel_view(
        df_u: &TotalView<U>,
        df_alpha: &TotalView<Alpha>,
    ) -> Result<Vec<Self>, PolarsError> {
        let unique_channel_u = df_u.column("Channel ID")?.u32()?.unique()?;
        let channels_u = unique_channel_u.iter().map(|i| i.unwrap());
        let unique_channel_alpha = df_alpha.column("Channel ID")?.u32()?.unique()?;
        let channels_alpha = unique_channel_alpha.iter().map(|i| i.unwrap());
        channels_u
            .zip(channels_alpha)
            .map(|(channel_u, channel_alpha)| {
                Ok(MergedLazyFrame(
                    concat_df_horizontal(
                        &[
                            df_u.view_by_channel_id(channel_u).collect()?,
                            df_alpha
                                .view_by_channel_id(channel_alpha)
                                .select([
                                    col(Alpha::nth_perturb_col_alias()),
                                    col(Alpha::delta_slope_col_alias()),
                                ])
                                .collect()?,
                        ],
                        false,
                    )?
                    .lazy(),
                ))
            })
            .collect::<Result<Vec<Self>, PolarsError>>()
    }
    /// Group values of same `U` and compute the average of `n1-nF_u` and `n1-nF_Alpha`
    pub fn view_mean(self) -> Result<DataFrame, PolarsError> {
        self.0
            // Group by "U"
            //┌─────┬────────────┬─────────────┐
            //│ U   ┆ n1-nF_U    ┆ n1-nF_Alpha │
            //│ --- ┆ ---        ┆ ---         │
            //│ i32 ┆ f64        ┆ f64         │
            //╞═════╪════════════╪═════════════╡
            //│ 0   ┆ 7.4593e-18 ┆ 31.308104   │
            //│ 2   ┆ 1.253893   ┆ 8.065987    │
            //│ 4   ┆ 2.824379   ┆ 2.05173     │
            //│ 6   ┆ 4.357392   ┆ 1.033147    │
            //│ 8   ┆ 4.234919   ┆ 8.160309    │
            //│ 10  ┆ 11.621275  ┆ 13.932687   │
            //│ 12  ┆ 6.091781   ┆ -2.91931    │
            //└─────┴────────────┴─────────────┘
            //
            .group_by_stable([col("U")])
            .agg([
                col(U::delta_slope_col_alias()).mean(),
                col(Alpha::delta_slope_col_alias()).mean(),
            ])
            .select([
                col("U"),
                col(U::delta_slope_col_alias()),
                col(Alpha::delta_slope_col_alias()),
            ])
            .collect()
    }
}

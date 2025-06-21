use std::marker::PhantomData;

use polars::{
    error::PolarsError,
    frame::DataFrame,
    functions::concat_df_horizontal,
    prelude::{IntoLazy, LazyFrame, col},
};

use crate::{Alpha, ChannelMeanView, JobType, U};

use super::{HubbardUPlot, Pipeline, ViewType, channel_view::ChannelView};

/// Represent the state that the dataframes are merged
/// In the pipeline, it means it merges a frame of `M` job type to self.
pub struct ConcatView<T, V, M, N>(
    PhantomData<T>,
    PhantomData<V>,
    PhantomData<M>,
    PhantomData<N>,
)
where
    T: JobType,
    V: JobType,
    M: ViewType<T>,
    N: ViewType<V>;

/// Type alias for convenience
/// `ConcatView<U, Alpha, ChannelView<U>, ChannelView<Alpha>>`
pub type ChannelMergedView = ConcatView<U, Alpha, ChannelView<U>, ChannelView<Alpha>>;

/// Type alias for convenience
/// `ConcatView<U, Alpha, ChannelView<U>, ChannelView<Alpha>>`
pub type ChannelMergedMeanView = ConcatView<U, Alpha, ChannelMeanView<U>, ChannelMeanView<Alpha>>;

impl<T, V, M, N> ViewType<T> for ConcatView<T, V, M, N>
where
    T: JobType,
    V: JobType,
    M: ViewType<T>,
    N: ViewType<V>,
{
}

/// We specifically concat U with Alpha, but not in the opposite order.
impl Pipeline<U, ChannelView<U>, LazyFrame> {
    /// Consumes self and `alpha`
    /// Our type system makes this happens self-explanatory
    pub fn concat_alpha(
        self,
        alpha: Pipeline<Alpha, ChannelView<Alpha>, LazyFrame>,
    ) -> Result<Pipeline<U, ChannelMergedView, LazyFrame>, PolarsError> {
        Ok(Pipeline::new(
            concat_df_horizontal(
                &[self.data.collect()?, alpha.to_be_merged().data.collect()?],
                false,
            )?
            .lazy(),
        ))
    }
}

impl Pipeline<U, ChannelMergedView, LazyFrame> {
    /// The finish line (or, currently) of our pipeline.
    /// Produces a channel_{id}_mean dataframe:
    ///┌─────┬────────────┬─────────────┐
    ///│ U   ┆ n1-nF_U    ┆ n1-nF_Alpha │
    ///│ --- ┆ ---        ┆ ---         │
    ///│ i32 ┆ f64        ┆ f64         │
    ///╞═════╪════════════╪═════════════╡
    ///│ 0   ┆ 7.4593e-18 ┆ 31.308104   │
    ///│ 2   ┆ 1.253893   ┆ 8.065987    │
    ///│ 4   ┆ 2.824379   ┆ 2.05173     │
    ///│ 6   ┆ 4.357392   ┆ 1.033147    │
    ///│ 8   ┆ 4.234919   ┆ 8.160309    │
    ///│ 10  ┆ 11.621275  ┆ 13.932687   │
    ///│ 12  ┆ 6.091781   ┆ -2.91931    │
    ///└─────┴────────────┴─────────────┘
    pub fn view_mean(self) -> Result<Pipeline<U, ChannelMergedMeanView, DataFrame>, PolarsError> {
        Ok(Pipeline::new(
            self.data
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
                .collect()?,
        ))
    }
}

impl HubbardUPlot for Pipeline<U, ChannelMergedMeanView, DataFrame> {
    type X = String;
    type Y = (String, Vec<f64>);

    fn xs(&self) -> Vec<Self::X> {
        self.data
            .column("U")
            .expect("The dataframe has column `U`")
            .i32()
            .expect("U value should be of type `i32`")
            .iter()
            .map(|i| format!("{}", i.expect("Must be i32")))
            .collect()
    }

    fn ys(&self) -> Vec<Self::Y> {
        vec![
            (
                U::delta_slope_col_alias(),
                self.data
                    .column(&U::delta_slope_col_alias())
                    .expect("The dataframe has column `n1-nF_U`")
                    .f64()
                    .expect("Must be f64")
                    .iter()
                    .flatten()
                    .collect::<Vec<f64>>(),
            ),
            (
                Alpha::delta_slope_col_alias(),
                self.data
                    .column(&Alpha::delta_slope_col_alias())
                    .expect("The dataframe has column `n1-nF_Alpha`")
                    .f64()
                    .expect("Must be f64")
                    .iter()
                    .flatten()
                    .collect::<Vec<f64>>(),
            ),
        ]
    }
}

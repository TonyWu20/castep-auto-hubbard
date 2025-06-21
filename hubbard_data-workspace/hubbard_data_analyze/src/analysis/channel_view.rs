use std::marker::PhantomData;

use polars::{
    error::PolarsError,
    frame::DataFrame,
    prelude::{LazyFrame, col},
};

use crate::job_type::{Alpha, JobType, U};

use super::{Pipeline, ViewColumn, ViewType};

pub mod mean_view;
pub use mean_view::ChannelMeanView;

/// The processed channel view will have the following columns:
/// ["Channel ID", "U", "u_pert/alpha_pert", "n1-nF_U/n1-nF_alpha"]
#[derive(Debug, Clone, Copy)]
pub struct ChannelView<T: JobType>(PhantomData<T>);

impl<T: JobType> ViewType<T> for ChannelView<T> {}

impl ViewColumn<U> for ChannelView<U> {
    /// ["Channel ID", "U", "u_pert", "n1-nF_U"]
    fn column_names() -> Vec<String> {
        vec!["Channel ID", "U", "u_pert", "n1-nF_U"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

impl ViewColumn<Alpha> for ChannelView<Alpha> {
    fn column_names() -> Vec<String> {
        vec!["Channel ID", "U", "alpha_pert", "n1-nF_Alpha"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

impl<T: JobType> Pipeline<T, ChannelView<T>, LazyFrame> {
    /// Aggregate the delta slope to mean value.
    pub fn to_mean_view(self) -> Result<Pipeline<T, ChannelMeanView<T>, DataFrame>, PolarsError> {
        Ok(Pipeline::new(
            self.data
                .group_by_stable([col("U")])
                .agg([col(T::delta_slope_col_alias()).mean()])
                .select([col("U"), col(T::delta_slope_col_alias())])
                .collect()?,
        ))
    }
}

impl Pipeline<Alpha, ChannelView<Alpha>, LazyFrame> {
    /// Generate the dataframe to be concatenated with U dataframe
    /// to build a merged channel view
    /// Only columns `["alpha_pert", "n1-nF_Alpha"]` stay
    /// This method consumes `self`
    pub fn to_be_merged(self) -> Self {
        Pipeline::new(self.data.select([
            col(Alpha::nth_perturb_col_alias()),
            col(Alpha::delta_slope_col_alias()),
        ]))
    }
}

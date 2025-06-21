use std::marker::PhantomData;

use polars::prelude::{ChunkUnique, LazyFrame, col, lit};

use crate::{Alpha, U, analysis::ViewType, job_type::JobType};

use super::{Pipeline, ViewColumn, channel_view::ChannelView};

#[derive(Debug, Copy, Clone)]
/// The processed total view will have the following columns:
/// [ "Channel ID", "U", "S1-S0", "SF-S0", "u/S1-S0 | alpha/s1-S0", "u/SF-S0 | alpha/SF-S0", "u_pert | alpha_pert", "n1-nF"]
pub struct TotalView<T: JobType>(PhantomData<T>);

impl<T: JobType> ViewType<T> for TotalView<T> {}

impl ViewColumn<U> for TotalView<U> {
    fn column_names() -> Vec<String> {
        vec![
            "Channel ID",
            "U",
            "S1-S0",
            "SF-S0",
            "u/S1-S0",
            "u/SF-S0",
            "u_pert",
            "n1-nF",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl ViewColumn<Alpha> for TotalView<Alpha> {
    fn column_names() -> Vec<String> {
        vec![
            "Channel ID",
            "U",
            "S1-S0",
            "SF-S0",
            "alpha/s1-S0",
            "alpha/SF-S0",
            "alpha_pert",
            "n1-nF",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl<T: JobType> Pipeline<T, TotalView<T>, LazyFrame> {
    /// From total to each channel
    /// Since we want to derive many channel views from one total view,
    /// we should clone the lazyframe for this action.
    pub fn to_channel_view(&self, channel_id: u32) -> Pipeline<T, ChannelView<T>, LazyFrame> {
        Pipeline::new(
            self.data
                .clone()
                .filter(col("Channel ID").eq(lit(channel_id)))
                .select([
                    col("Channel ID"),
                    col("U"),
                    col(T::nth_perturb_col_alias()),
                    col("n1-nF").alias(T::delta_slope_col_alias()),
                ]),
        )
    }

    /// Get the list of unique channel ids in the frame.
    pub fn channels(&self) -> Vec<u32> {
        self.data
            .clone()
            .select([col("Channel ID")])
            .collect()
            .and_then(|dataframe| {
                dataframe
                    .column("Channel ID")
                    .and_then(|col| col.u32())
                    .and_then(|col| col.unique())
            })
            .map(|col| col.iter().flatten().collect::<Vec<u32>>())
            .unwrap()
    }
}

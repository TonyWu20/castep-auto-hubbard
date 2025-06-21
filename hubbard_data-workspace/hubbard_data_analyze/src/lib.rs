#![allow(dead_code)]
#![warn(missing_docs)]
//! This is an internal crate for `hubbard_data`.
//! It offers the utility functions to sort out the data in input result csvs (generated from `auto_hubbard`)
//! and returns the desired dataframes, which will be output to `csv` files or directed to `hubbard_data_plot`
//! for visualization.
mod analysis;
mod job_type;

pub use analysis::{
    HubbardUPlot, Pipeline,
    channel_view::{ChannelMeanView, ChannelView},
    csv_path::CSVPath,
    merged_view::{ChannelMergedMeanView, ChannelMergedView},
    total_view::TotalView,
};
pub use job_type::{Alpha, JobType, U};
pub use polars::io::SerWriter;
pub use polars::prelude::{CsvWriter, DataFrame};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use polars::{error::PolarsError, frame::DataFrame};

    use crate::{Alpha, ChannelMergedMeanView, JobType, Pipeline, U};

    #[test]
    fn it_works() {
        let result_folder = Path::new("../../sorting");

        let result_df_u = U::csv_path(result_folder).process_data(0.05).unwrap();
        println!("U");
        let result_df_alpha = Alpha::csv_path(result_folder).process_data(0.05).unwrap();
        println!("Alpha");
        let channels_u = result_df_u.channels();
        let channels_alpha = result_df_alpha.channels();
        channels_u
            .iter()
            .zip(channels_alpha.iter())
            .map(
                |(&chan_u, &chan_alpha)| -> Result<
                    Pipeline<U, ChannelMergedMeanView, DataFrame>,
                    PolarsError,
                > {
                    result_df_u
                        .to_channel_view(chan_u)
                        .concat_alpha(result_df_alpha.to_channel_view(chan_alpha))?
                    .view_mean()

                },
            )
            .try_for_each(|result_view| {
                println!("{}", result_view?.data());
                Ok::<(), PolarsError>(())
            }).expect("Pipeline demonstration fail");
    }
}

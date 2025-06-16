#![allow(dead_code)]
#![warn(missing_docs)]
//! This is an internal crate for `hubbard_data`.
//! It offers the utility functions to sort out the data in input result csvs (generated from `auto_hubbard`)
//! and returns the desired dataframes, which will be output to `csv` files or directed to `hubbard_data_plot`
//! for visualization.
mod analysis;
mod job_type;

pub use analysis::channel_view::{MergedLazyChannel, Plottable, mean_view::ChannelMeanView};
pub use analysis::total_view::{CsvPath, TotalView};
pub use job_type::{Alpha, JobType, U};
pub use polars::io::SerWriter;
pub use polars::prelude::{CsvWriter, DataFrame};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        analysis::channel_view::MergedLazyChannel,
        job_type::{Alpha, JobType, U},
    };
    #[test]
    fn it_works() {
        let result_folder = Path::new("../../sorting");

        let result_df_u = U::csv_path(result_folder).process_data(0.05).unwrap();
        println!("U");
        println!("{}", *result_df_u);
        let result_df_alpha = Alpha::csv_path(result_folder).process_data(0.05).unwrap();
        println!("Alpha");
        println!("{}", *result_df_alpha);
        MergedLazyChannel::merge_u_alpha_channel_view(&result_df_u, &result_df_alpha)
            .unwrap()
            .into_iter()
            .for_each(|merged| println!("{}", merged.view_mean().unwrap()));
    }
}

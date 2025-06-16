use std::fs::{File, create_dir_all};

use hubbard_data_analyze::{Alpha, CsvWriter, JobType, MergedLazyChannel, Plottable, SerWriter, U};
use hubbard_data_args::{HubbardDataCli, Parser};
use hubbard_data_plot::plot_channel_mean;

fn main() -> Result<(), anyhow::Error> {
    let cli = HubbardDataCli::parse();
    let src_dir = cli.result_folder();
    let df_u = U::csv_path(src_dir).process_data(cli.u_perturb_val())?;
    let df_alpha = Alpha::csv_path(src_dir).process_data(cli.alpha_perturb_val())?;
    let merged = MergedLazyChannel::merge_u_alpha_channel_view(&df_u, &df_alpha)?;
    let dest_dir = src_dir.join("plot");
    // Create the `dest_dir`, and don't care about the result.
    // If result is Ok(()) then the directory is successfully created, or it has already existed.
    create_dir_all(&dest_dir).ok();
    merged
        .into_iter()
        .map(|m| m.view_mean())
        .enumerate()
        .try_for_each(|(i, m)| {
            let mut mean_view = m?;
            let file = File::create(dest_dir.join(format!("channel_{}_mean.csv", i + 1)))?;
            // `DerefMut` of `ChannelMeanView`
            CsvWriter::new(file).finish(&mut mean_view)?;
            plot_channel_mean(
                &mean_view.xs(),
                &mean_view.ys().try_into().unwrap(),
                i as u32,
                &dest_dir,
            )?;
            Ok::<(), anyhow::Error>(())
        })
}

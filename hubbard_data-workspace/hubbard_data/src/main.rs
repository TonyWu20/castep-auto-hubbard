use std::{
    fs::{File, create_dir_all},
    iter::Map,
};

use hubbard_data_analyze::{
    Alpha, ChannelMeanView, CsvWriter, DataFrame, HubbardUPlot, JobType, Pipeline, SerWriter, U,
};
use hubbard_data_args::{HubbardDataCli, Parser};
use hubbard_data_plot::plot_channel_mean;

/// Can run for both U and Alpha results or U results only.
/// Since I don't have the castep code that can handle `HUBBARD_ALPHA` section, leave the Alpha-single run mode unimplemented
fn main() -> Result<(), anyhow::Error> {
    let cli = HubbardDataCli::parse();
    match cli.mode().unwrap_or_default() {
        hubbard_data_args::Mode::Both => analyze_both(&cli),
        hubbard_data_args::Mode::U => {
            let dest_dir = cli.result_folder().join(format!("plot_{}", U::job_type()));
            create_dir_all(&dest_dir).ok();
            analyze_one_frame::<U>(&cli)?.try_for_each(|result| {
                let (channel_id, mut df_mean) = result?;
                let file =
                    File::create(dest_dir.join(format!("channel_{}_mean_U.csv", channel_id)))?;
                CsvWriter::new(file).finish(df_mean.data_mut())?;
                let (xs, ys) = (df_mean.xs(), df_mean.ys());
                plot_channel_mean(&xs, &ys, channel_id, &dest_dir)?;
                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        }
        hubbard_data_args::Mode::Alpha => unimplemented!(),
    }
}

/// Main function for analyzing both csv together.
fn analyze_both(cli: &HubbardDataCli) -> Result<(), anyhow::Error> {
    let src_dir = cli.result_folder();
    let df_u = U::csv_path(src_dir).process_data(cli.u_perturb_val())?;
    let df_alpha = Alpha::csv_path(src_dir).process_data(cli.alpha_perturb_val())?;
    let dest_dir = src_dir.join("plot");
    create_dir_all(&dest_dir).ok();
    let channels_u = df_u.channels();
    let channels_alpha = df_alpha.channels();
    channels_u
        .iter()
        .zip(channels_alpha.iter())
        .map(|(&c_u, &c_a)| {
            (
                c_u,
                df_u.to_channel_view(c_u)
                    .concat_alpha(df_alpha.to_channel_view(c_a).to_be_merged())
                    .and_then(|concated| concated.view_mean()),
            )
        })
        .try_for_each(|res_concat_mean| {
            let (c_u, concat_mean) = res_concat_mean;
            let mut concat_mean = concat_mean?;
            let file = File::create(dest_dir.join(format!("channel_{}_mean.csv", c_u)))?;
            CsvWriter::new(file).finish(concat_mean.data_mut())?;
            plot_channel_mean(&concat_mean.xs(), &concat_mean.ys(), c_u, &dest_dir)?;
            Ok::<(), anyhow::Error>(())
        })?;
    Ok(())
}

#[allow(clippy::type_complexity)]
/// Directly return the iterator, prevent early collect
fn analyze_one_frame<'a, T: JobType + 'a>(
    cli: &'a HubbardDataCli,
) -> Result<
    Map<
        std::vec::IntoIter<u32>,
        impl FnMut(u32) -> Result<(u32, Pipeline<T, ChannelMeanView<T>, DataFrame>), anyhow::Error>,
    >,
    anyhow::Error,
> {
    let df =
        T::csv_path(cli.result_folder()).process_data(cli.perturb_value().try_into_single()?)?;
    Ok(df
        .channels()
        .into_iter()
        .map(move |i| Ok((i, df.to_channel_view(i).to_mean_view()?))))
}

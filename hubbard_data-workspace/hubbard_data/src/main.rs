use std::{env, fs::File};

use hubbard_data_analyze::{
    CsvWriter, JobType, SerWriter, get_response, merge_u_alpha_channel_view, view_by_channel_id,
    view_mean,
};
use hubbard_data_args::{HubbardDataCli, Parser};

fn main() -> Result<(), anyhow::Error> {
    let cli = HubbardDataCli::parse();
    let u_df = get_response(
        cli.result_folder().unwrap_or(&env::current_dir()?),
        JobType::U,
        cli.u_perturb_val(),
    )?;
    let alpha_df = get_response(
        cli.result_folder().unwrap_or(&env::current_dir()?),
        JobType::Alpha,
        cli.alpha_perturb_val(),
    )?;
    u_df.column("Channel ID")?
        .unique_stable()?
        .u32()?
        .iter()
        .map(|i| i.unwrap())
        .try_for_each(|i| {
            let mut channel_mean_df = merge_u_alpha_channel_view(
                view_by_channel_id(&u_df, i, JobType::U),
                view_by_channel_id(&alpha_df, i, JobType::Alpha),
            )
            .and_then(view_mean)?;
            let channel_mean_csv = File::create(format!("channel_{}_mean.csv", i))?;
            CsvWriter::new(channel_mean_csv).finish(&mut channel_mean_df)?;
            Ok::<(), anyhow::Error>(())
        })?;
    todo!()
}

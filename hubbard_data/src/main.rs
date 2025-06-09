use std::{
    env,
    path::{Path, PathBuf},
};

use catppuccin::Rgb;
use clap::Parser;
use dataframe::{get_result, plot::plot_channel_mean, view_by_channel_id, view_mean};
use plotters::style::RGBColor;
use polars::{frame::DataFrame, io::SerWriter, prelude::CsvWriter};

mod config;
mod dataframe;

#[derive(Debug, clap::Parser)]
/// Sorting data of `result_u_final.csv` and `result_alpha_final.csv`
pub struct Args {
    #[arg(short = 's')]
    pub(crate) result_folder: Option<PathBuf>,
    pub(crate) perturb_by_val: f64,
    #[arg(short, long)]
    pub(crate) verbose: Option<bool>,
}

fn write_view_by_channel_id(
    df_u: &DataFrame,
    df_alpha: &DataFrame,
    result_folder: &Path,
) -> Result<(), anyhow::Error> {
    let ids = df_u.column("Channel ID").unwrap().unique_stable().unwrap();
    ids.u32()?.iter().try_for_each(|i| {
        let channel_id =
            i.expect("Channel ID is guaranteed to exist and be within the scope of `u32`");
        let result_ly = view_by_channel_id(df_u, df_alpha, channel_id)?;
        let mut result = result_ly.clone().collect()?;
        let mut file = std::fs::File::create(
            result_folder.join(format!("channel_{}_sorted.csv", channel_id)),
        )?;
        let mut result_mean = view_mean(result_ly)?;
        println!("{}", &result_mean);
        let mut file_mean = std::fs::File::create(
            result_folder.join(format!("channel_{}_sorted_mean.csv", channel_id)),
        )?;
        let Rgb { r, g, b } = catppuccin::PALETTE.latte.colors.lavender.rgb;
        let lavender = RGBColor(r, g, b);
        let Rgb { r, g, b } = catppuccin::PALETTE.latte.colors.maroon.rgb;
        let maroon = RGBColor(r, g, b);
        plot_channel_mean(&result_mean, channel_id, result_folder, lavender, maroon)?;
        CsvWriter::new(&mut file).finish(&mut result)?;
        CsvWriter::new(&mut file_mean).finish(&mut result_mean)?;
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let result_folder = args.result_folder.unwrap_or(env::current_dir()?);
    let perturb_val = args.perturb_by_val;
    // Sort U
    let mut result_df_u = get_result(&result_folder, perturb_val, crate::config::JobType::U)?;
    if args.verbose.is_some_and(|b| b) {
        println!("Result of U perturbation:");
        println!("{}", result_df_u);
    }
    let mut result_file_u = std::fs::File::create(result_folder.join("sorted_u.csv"))?;
    CsvWriter::new(&mut result_file_u).finish(&mut result_df_u)?;
    // Sort Alpha
    let mut result_df_alpha = get_result(&result_folder, perturb_val, config::JobType::Alpha)?;
    let mut result_file_alpha = std::fs::File::create(result_folder.join("sorted_alpha.csv"))?;
    if args.verbose.is_some_and(|b| b) {
        println!("Result of Alpha perturbation:");
        println!("{}", result_df_alpha);
    }
    CsvWriter::new(&mut result_file_alpha).finish(&mut result_df_alpha)?;
    // Write out each channel's result
    write_view_by_channel_id(&result_df_u, &result_df_alpha, &result_folder)?;
    Ok(())
}

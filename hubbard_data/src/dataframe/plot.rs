use std::path::Path;

use charming::ImageRenderer;
use charming::component::{Axis, Legend, Title};
use charming::element::font_settings::{FontFamily, FontWeight};
use charming::element::{AxisLabel, Orient, TextStyle};
use charming::series::Line;
use polars::frame::DataFrame;

pub fn plot_channel_mean<P: AsRef<Path>>(
    channel_mean_df: &DataFrame,
    channel_id: u32,
    result_folder: P,
) -> Result<(), anyhow::Error> {
    let ys_u = channel_mean_df
        .column("n1-nF_U")?
        .f64()?
        .iter()
        .map(|f| f.expect("Must be f64"))
        .collect::<Vec<f64>>();
    let ys_alpha = channel_mean_df
        .column("n1-nF_Alpha")?
        .f64()?
        .iter()
        .map(|f| f.expect("Must be f64"))
        .collect::<Vec<f64>>();
    let chart = charming::Chart::new()
        .title(
            Title::new()
                .text(format!("Channel ID: {}", channel_id))
                .left("center")
                .text_style(
                    TextStyle::new()
                        .font_weight(FontWeight::Bolder)
                        .font_family(FontFamily::SansSerif)
                        .font_size(36),
                ),
        )
        .x_axis(
            Axis::new()
                .data(
                    channel_mean_df
                        .column("U")?
                        .i32()?
                        .iter()
                        .map(|i| format!("{}", i.expect("Must be i32")))
                        .collect::<Vec<String>>(),
                )
                .axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(16)),
        )
        .y_axis(
            Axis::new().axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(16)),
        )
        .legend(
            Legend::new()
                .data(vec!["n1-nF_U", "n1-nF_Alpha"])
                .orient(Orient::Vertical)
                .right(10)
                .text_style(TextStyle::new().font_weight(FontWeight::Bold).font_size(14)),
        )
        .series(Line::new().name("n1-nF_U").data(ys_u))
        .series(Line::new().name("n1-nF_Alpha").data(ys_alpha));
    let mut renderer = ImageRenderer::new(1024, 768);
    renderer.save_format(
        charming::ImageFormat::Png,
        &chart,
        result_folder
            .as_ref()
            .join(format!("channel_{}_mean.png", channel_id)),
    )?;
    Ok(())
}

// With `plotlars`
// pub fn plot_channel_mean(channel_mean_df: &DataFrame, channel_id: u32) -> LinePlot {
//     LinePlot::builder()
//         .data(channel_mean_df)
//         .x("U")
//         .y("n1-nF_U")
//         .additional_lines(vec!["n1-nF_Alpha"])
//         .colors(vec![Rgb(220, 20, 20), Rgb(20, 20, 250)])
//         .width(3.0)
//         .with_shape(true)
//         .plot_title(
//             Text::from(format!("Channel ID: {}", channel_id))
//                 .font("Arial")
//                 .size(24),
//         )
//         .legend_title(Text::from("Series").font("Arial").size(14))
//         .x_axis(
//             &Axis::new()
//                 .tick_direction(plotlars::TickDirection::OutSide)
//                 .tick_values(
//                     (0..=12)
//                         .filter(|v| v % 2 == 0)
//                         .map(|v| v as f64)
//                         .collect::<Vec<f64>>(),
//                 )
//                 .tick_labels(
//                     (0..=12)
//                         .filter(|v| v % 2 == 0)
//                         .map(|v| format!("{v}"))
//                         .collect::<Vec<String>>(),
//                 ),
//         )
//         .build()
// }

// With `plotters`
// pub fn plot_channel_mean<P: AsRef<Path>>(
//     channel_mean_df: &DataFrame,
//     channel_id: u32,
//     result_folder: P,
//     u_color: RGBColor,
//     alpha_color: RGBColor,
// ) -> Result<(), anyhow::Error> {
//     // Set image backend
//     let output = result_folder
//         .as_ref()
//         .join(format!("channel_{}_mean.svg", channel_id));

//     register_font(
//         "sans-serif bold",
//         plotters::style::FontStyle::Normal,
//         include_bytes!("../../SourceSans3-Bold.otf"),
//     )
//     .unwrap_or_default();

//     let root = plotters::backend::SVGBackend::new(&output, (1280, 800)).into_drawing_area();
//     let Rgb { r, g, b } = catppuccin::PALETTE.latte.colors.base.rgb;
//     root.fill(&RGBColor(r, g, b))?;

//     let xs = channel_mean_df
//         .column("U")?
//         .i32()?
//         .iter()
//         .map(|i| i.expect("Must be i32"))
//         .collect::<Vec<i32>>();

//     let x_spec = *xs.iter().min().unwrap() as f64 - 0.5..*xs.iter().max().unwrap() as f64 + 0.5;

//     let y_u = channel_mean_df
//         .column("n1-nF_U")?
//         .f64()?
//         .iter()
//         .map(|f| f.expect("y must be `f64`"))
//         .collect::<Vec<f64>>();
//     let y_alpha = channel_mean_df
//         .column("n1-nF_Alpha")?
//         .f64()?
//         .iter()
//         .map(|f| f.expect("y must be `f64`"))
//         .collect::<Vec<f64>>();
//     let y_spec = f64::min(
//         y_u.clone().into_iter().reduce(f64::min).unwrap(),
//         y_alpha.clone().into_iter().reduce(f64::min).unwrap(),
//     ) - 0.5
//         ..f64::max(
//             y_u.clone().into_iter().reduce(f64::max).unwrap(),
//             y_alpha.clone().into_iter().reduce(f64::max).unwrap(),
//         ) + 0.5;

//     // Configure chart
//     let mut chart = ChartBuilder::on(&root)
//         .caption(
//             format!("Channel ID: {}", channel_id),
//             ("sans-serif bold", 48).into_font(),
//         )
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_cartesian_2d(x_spec, y_spec)?;
//     chart.configure_mesh().disable_mesh().draw()?;
//     chart
//         .draw_series(
//             LineSeries::new(
//                 xs.iter().zip(y_u.iter()).map(|(x, y)| (*x as f64, *y)),
//                 u_color.stroke_width(5).filled(),
//             )
//             .point_size(10),
//         )?
//         .label("n1-nF_U")
//         .legend(|(x, y)| Rectangle::new([(x - 15, y + 3), (x, y - 3)], u_color.filled()));
//     chart
//         .draw_series(
//             LineSeries::new(
//                 xs.iter().zip(y_alpha.iter()).map(|(x, y)| (*x as f64, *y)),
//                 alpha_color.stroke_width(5).filled(),
//             )
//             .point_size(10),
//         )?
//         .label("n1-nF_alpha")
//         .legend(|(x, y)| Rectangle::new([(x - 15, y + 3), (x, y - 3)], alpha_color.filled()));

//     chart
//         .configure_series_labels()
//         .label_font(("sans-serif bold", 32))
//         .position(plotters::chart::SeriesLabelPosition::UpperRight)
//         .draw()?;
//     root.present()?;
//     Ok(())
// }

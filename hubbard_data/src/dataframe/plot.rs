use plotlars::{Axis, LinePlot, Plot, Rgb, Text};
use polars::frame::DataFrame;

pub fn plot_channel_mean(channel_mean_df: &DataFrame, channel_id: u32) -> LinePlot {
    LinePlot::builder()
        .data(channel_mean_df)
        .x("U")
        .y("n1-nF_U")
        .additional_lines(vec!["n1-nF_Alpha"])
        .colors(vec![Rgb(220, 20, 20), Rgb(20, 20, 250)])
        .width(3.0)
        .with_shape(true)
        .plot_title(
            Text::from(format!("Channel ID: {}", channel_id))
                .font("Arial")
                .size(24),
        )
        .legend_title(Text::from("Series").font("Arial").size(14))
        .x_axis(
            &Axis::new()
                .tick_direction(plotlars::TickDirection::OutSide)
                .tick_values(
                    (0..=12)
                        .filter(|v| v % 2 == 0)
                        .map(|v| v as f64)
                        .collect::<Vec<f64>>(),
                )
                .tick_labels(
                    (0..=12)
                        .filter(|v| v % 2 == 0)
                        .map(|v| format!("{v}"))
                        .collect::<Vec<String>>(),
                ),
        )
        .build()
}

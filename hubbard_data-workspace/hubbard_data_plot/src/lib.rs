use std::path::Path;

use charming::{
    Chart, EchartsError, ImageRenderer,
    component::{Axis, Legend, Title},
    element::{
        AxisLabel, Orient, TextStyle,
        font_settings::{FontFamily, FontWeight},
    },
    series::Line,
};

pub fn plot_channel_mean<P: AsRef<Path>>(
    xs: &[String],
    ys: &[(String, Vec<f64>); 2],
    channel_id: u32,
    result_folder: P,
) -> Result<(), EchartsError> {
    let chart = charming::Chart::new()
        .title(
            Title::new()
                .text(format!("Channel ID: {}", channel_id))
                .left("center")
                .text_style(
                    TextStyle::new()
                        .font_weight(FontWeight::Bolder)
                        .font_family(FontFamily::SansSerif)
                        .font_size(28),
                ),
        )
        .x_axis(
            Axis::new()
                .data(xs.to_vec())
                .axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(14)),
        )
        .y_axis(
            Axis::new().axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(14)),
        )
        .legend(
            Legend::new()
                .data(vec!["n1-nF_U", "n1-nF_Alpha"])
                .orient(Orient::Vertical)
                .right(10)
                .text_style(TextStyle::new().font_weight(FontWeight::Bold).font_size(14)),
        );
    let chart_with_series = ys
        .iter()
        .map(|(name, y)| Line::new().name(name.to_string()).data(y.to_vec()))
        .fold(chart, |acc: Chart, line| acc.series(line));
    let mut renderer = ImageRenderer::new(800, 600);
    renderer.save_format(
        charming::ImageFormat::Png,
        &chart_with_series,
        result_folder
            .as_ref()
            .join(format!("channel_{}_mean.png", channel_id)),
    )
}

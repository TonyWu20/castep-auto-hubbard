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

/// Interface struct
pub struct PlotHub<'a, P: AsRef<Path>> {
    xs: &'a [String],
    ys: &'a [(String, Vec<f64>)],
    channel_id: u32,
    result_folder: P,
}

impl<'a, P: AsRef<Path>> PlotHub<'a, P> {
    pub fn new(
        xs: &'a [String],
        ys: &'a [(String, Vec<f64>)],
        channel_id: u32,
        result_folder: P,
    ) -> Self {
        Self {
            xs,
            ys,
            channel_id,
            result_folder,
        }
    }

    pub fn plot_channel_mean(&self) -> Result<(), EchartsError> {
        let chart = charming::Chart::new()
            .title(
                Title::new()
                    .text(format!("Channel ID: {}", self.channel_id))
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
                    .data(self.xs.to_vec())
                    .axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(14)),
            )
            .y_axis(
                Axis::new()
                    .axis_label(AxisLabel::new().font_weight(FontWeight::Bold).font_size(14)),
            )
            .legend(
                Legend::new()
                    .data(vec!["n1-nF_U", "n1-nF_Alpha"])
                    .orient(Orient::Vertical)
                    .right(10)
                    .text_style(TextStyle::new().font_weight(FontWeight::Bold).font_size(14)),
            );
        let chart_with_series = self
            .ys
            .iter()
            .map(|(name, y)| Line::new().name(name.to_string()).data(y.to_vec()))
            .fold(chart, |acc: Chart, line| acc.series(line));
        let mut renderer = ImageRenderer::new(800, 600);
        renderer.save_format(
            charming::ImageFormat::Png,
            &chart_with_series,
            self.result_folder
                .as_ref()
                .join(format!("channel_{}_mean.png", self.channel_id)),
        )
    }
}

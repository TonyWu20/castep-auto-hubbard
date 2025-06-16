use polars::frame::DataFrame;

use super::Plottable;

#[derive(Debug, Clone)]
/// A dataframe that computes the mean values of the U/Alpha perturbation responses at each U value
/// for each channel
pub struct ChannelMeanView(pub(crate) DataFrame);

impl std::ops::DerefMut for ChannelMeanView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for ChannelMeanView {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ChannelMeanView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <DataFrame as std::fmt::Display>::fmt(&self.0, f)
    }
}

impl Plottable for ChannelMeanView {
    type X = String;
    type Y = (String, Vec<f64>);
    fn xs(&self) -> Vec<Self::X> {
        self.0
            .column("U")
            .expect("The dataframe has column `U`")
            .i32()
            .expect("U value should be of type `i32`")
            .iter()
            .map(|i| format!("{}", i.expect("Must be i32")))
            .collect()
    }

    fn ys(&self) -> Vec<Self::Y> {
        vec![
            (
                "n1-nF_U".to_string(),
                self.0
                    .column("n1-nF_U")
                    .expect("The dataframe has column `n1-nF_U`")
                    .f64()
                    .expect("Must be f64")
                    .iter()
                    .flatten()
                    .collect::<Vec<f64>>(),
            ),
            (
                "n1-nF_Alpha".to_string(),
                self.0
                    .column("n1-nF_Alpha")
                    .expect("The dataframe has column `n1-nF_Alpha`")
                    .f64()
                    .expect("Must be f64")
                    .iter()
                    .flatten()
                    .collect::<Vec<f64>>(),
            ),
        ]
    }
}

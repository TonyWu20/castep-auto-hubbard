use std::marker::PhantomData;

use polars::frame::DataFrame;

use crate::{
    Alpha, JobType, U,
    analysis::{HubbardUPlot, Pipeline, ViewColumn, ViewType},
};

#[derive(Debug, Clone)]
/// A dataframe that computes the mean values of the U/Alpha perturbation responses at each U value
/// for each channel
pub struct ChannelMeanView<T: JobType>(PhantomData<T>);

impl ViewColumn<U> for ChannelMeanView<U> {
    fn column_names() -> Vec<String> {
        vec!["U", "n1-nF_U"].into_iter().map(String::from).collect()
    }
}

impl ViewColumn<Alpha> for ChannelMeanView<Alpha> {
    fn column_names() -> Vec<String> {
        vec!["U", "n1-nF_Alpha"]
            .into_iter()
            .map(String::from)
            .collect()
    }
}

impl<T: JobType> ViewType<T> for ChannelMeanView<T> {}

impl HubbardUPlot for Pipeline<U, ChannelMeanView<U>, DataFrame> {
    type X = String;
    type Y = (String, Vec<f64>);
    fn xs(&self) -> Vec<Self::X> {
        self.data
            .column("U")
            .expect("The dataframe has column `U`")
            .i32()
            .expect("U value should be of type `i32`")
            .iter()
            .map(|i| format!("{}", i.expect("Must be i32")))
            .collect()
    }

    fn ys(&self) -> Vec<Self::Y> {
        vec![(
            U::delta_slope_col_alias(),
            self.data
                .column(&U::delta_slope_col_alias())
                .expect("The dataframe has column `n1-nF_U`")
                .f64()
                .expect("Must be f64")
                .iter()
                .flatten()
                .collect::<Vec<f64>>(),
        )]
    }
}

impl HubbardUPlot for Pipeline<Alpha, ChannelMeanView<Alpha>, DataFrame> {
    type X = String;
    type Y = (String, Vec<f64>);
    fn xs(&self) -> Vec<Self::X> {
        self.data
            .column("U")
            .expect("The dataframe has column `U`")
            .i32()
            .expect("U value should be of type `i32`")
            .iter()
            .map(|i| format!("{}", i.expect("Must be i32")))
            .collect()
    }

    fn ys(&self) -> Vec<Self::Y> {
        vec![(
            Alpha::delta_slope_col_alias(),
            self.data
                .column(&Alpha::delta_slope_col_alias())
                .expect("The dataframe has column `n1-nF_Alpha`")
                .f64()
                .expect("Must be f64")
                .iter()
                .flatten()
                .collect::<Vec<f64>>(),
        )]
    }
}

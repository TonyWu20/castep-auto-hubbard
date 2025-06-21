use std::{fmt::Debug, marker::PhantomData};

use crate::JobType;

pub mod channel_view;
pub mod csv_path;
pub mod merged_view;
pub mod total_view;

/// A trait to represent the type indicates the current view type
/// of `PipeLine`
pub trait ViewType<T: JobType> {}

pub trait ViewColumn<T: JobType> {
    /// In case we get lost in the midst of codes and dataframe operations,
    /// this method serves as a reminder for the column names of the current view.
    /// It is also suggested that every struct implementing `ViewType` should write its
    /// column names in their doc comments.
    fn column_names() -> Vec<String> {
        unimplemented!()
    }
}

#[derive(Clone)]
/// A struct to represent our operations on the lazyframe
pub struct Pipeline<T: JobType, U: ViewType<T>, D: Clone> {
    job_type: PhantomData<T>,
    view_type: PhantomData<U>,
    data: D,
}

impl<T: JobType, U: ViewType<T>, D: Clone + Debug> Debug for Pipeline<T, U, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("job_type", &self.job_type)
            .field("view_type", &self.view_type)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: JobType, U: ViewType<T>, D: Clone> Pipeline<T, U, D> {
    /// Create a pipeline
    pub fn new(data: D) -> Self {
        Self {
            job_type: PhantomData,
            view_type: PhantomData,
            data,
        }
    }

    /// Get the data
    pub fn data(&self) -> &D {
        &self.data
    }
}

/// To be able to plot with `hubbard_data_plot`
pub trait HubbardUPlot {
    /// Type for X axis in corresponding plotting lib
    type X;
    /// Type for Y axis in corresponding plotting lib
    type Y;
    /// XAxis data representation
    fn xs(&self) -> Vec<Self::X>;
    /// YAxis data representation
    fn ys(&self) -> Vec<Self::Y>;
}

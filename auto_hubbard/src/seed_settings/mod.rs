use std::{
    fmt::{Display, Write},
    path::Display,
    str::FromStr,
};

use clap::ValueEnum;

mod cell_setup;
mod param_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum JobType {
    U,
    Alpha,
}

impl Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::U => f.write_str("u"),
            JobType::Alpha => f.write_str("alpha"),
        }
    }
}

#[derive(Debug)]
pub struct JobTypeParsingError;

impl Display for JobTypeParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid input of JobType (u/U/alpha/Alpha)")
    }
}

impl std::error::Error for JobTypeParsingError {}

impl FromStr for JobType {
    type Err = JobTypeParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "u" | "U" => Ok(Self::U),
            "alpha" | "Alpha" => Ok(Self::Alpha),
            _ => Err(JobTypeParsingError),
        }
    }
}

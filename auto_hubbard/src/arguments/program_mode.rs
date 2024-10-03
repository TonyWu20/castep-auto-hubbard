use std::fmt::Display;

use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ProgramMode {
    Serial,
    Parallel,
    Read,
}

impl Display for ProgramMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramMode::Serial => f.write_str("serial"),
            ProgramMode::Parallel => f.write_str("parallel"),
            ProgramMode::Read => f.write_str("read"),
        }
    }
}

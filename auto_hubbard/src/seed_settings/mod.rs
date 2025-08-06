mod cell_setup;
mod job_type;
mod param_setup;
mod private;
mod stage;
mod hubbard_value {

    pub trait HubbardJob {
        fn perturbed_alpha(&self) -> f64;
        fn u_block_value(&self) -> f64;
    }

    pub enum HubbardValue {
        U(f64, f64),
        Alpha(f64, f64),
    }

    impl HubbardValue {
        pub fn u_block_value(&self) -> f64 {
            match self {
                HubbardValue::U(u, _) => *u,
                HubbardValue::Alpha(u, _) => *u,
            }
        }
    }
}

pub use cell_setup::{CellFile, HubbardUCell};
pub use job_type::{JobType, JobTypeParsingError};
pub use param_setup::{HubbardUParam, ParamFile};
pub use stage::{BeforePerturb, Init, Perturbed, Stage};

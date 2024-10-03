use std::process::exit;

use arguments::Cli;
use clap::Parser;
use inquire::{
    required,
    validator::{ErrorMessage, Validation},
    CustomType,
};

use crate::arguments::ReadArgs;

mod arguments;
mod errors;
mod seed_settings;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        arguments::JobCommands::Read(args) => {
            #[cfg(debug_assertions)]
            {
                println!("{}", args.result_path);
                let mut args = args.clone();
                dbg!(args.set_from_folder_name().is_ok());
                dbg!(args);
            }
            let mut args = args.clone();
            let set = args.set_from_folder_name();
            if let Err(e) = set {
                println!("{e}");
                args = CustomType::<ReadArgs>::new("Please enter the name of the result folder (e.g.: XXX_[jobtype]_[init_input_u]_[step_u]_[final_u]_[perturb_init]_[perturb_step]_[perturb_final]_STEPS_[perturb_times])")
                    .prompt().unwrap();
                args.invoke()
            }
            args.invoke()
        }
        arguments::JobCommands::Calc(_args) => todo!(),
    }
}

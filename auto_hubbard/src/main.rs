use std::io;

use arguments::Cli;
use clap::Parser;
mod arguments;
mod errors;
mod seed_settings;

fn main() -> Result<(), io::Error> {
    let mut cli = Cli::parse();
    match &mut cli.command_mut() {
        arguments::JobCommands::Read(args) => {
            #[cfg(debug_assertions)]
            {
                println!("{}", args.result_path);
                dbg!(args.set_from_folder_name().is_ok());
                dbg!(args);
                Ok(())
            }
            #[cfg(not(debug_assertions))]
            {
                use crate::arguments::ReadArgs;
                use inquire::CustomType;
                let set = args.set_from_folder_name();
                if let Err(e) = set {
                    println!("{e}");
                    let new_args = CustomType::<ReadArgs>::new("Please enter the name of the result folder (e.g.: XXX_[jobtype]_[init_input_u]_[step_u]_[final_u]_[perturb_init]_[perturb_step]_[perturb_final]_STEPS_[perturb_times])")
                    .prompt().unwrap();
                    new_args.invoke()
                }
                args.invoke()
            }
        }
        arguments::JobCommands::Calc(calc_args) => calc_args.invoke(),
    }
}

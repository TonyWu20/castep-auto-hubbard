use std::process::exit;

use arguments::Cli;
use clap::{Command, Parser};

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
                exit(0)
            }
            args.invoke()
        }
        arguments::JobCommands::Calc(args) => todo!(),
    }
}

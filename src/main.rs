mod cmd;
mod config;
mod data;
mod syscommand;

use std::process::ExitCode;

use crate::cmd::Cmd;

fn main() -> ExitCode {
    match Cmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("An error has occurred: {}", e);
            ExitCode::FAILURE
        }
    }
}

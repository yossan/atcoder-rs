mod cmd;
mod syscommand;

use std::process::ExitCode;

use crate::cmd::Cmd;

fn main() -> ExitCode {
    match Cmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_e) => ExitCode::FAILURE,
    }
}

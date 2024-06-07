mod new;
mod testcase;

use clap::Parser;
use new::New;
use std::error::Error;
use testcase::Testcase;

pub trait Run {
    fn run(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cmd {
    New(New),
    Testcase(Testcase),
}

impl Cmd {
    pub fn parse() -> Cmd {
        clap::Parser::parse()
    }
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Cmd::New(cmd) => cmd.run(),
            Cmd::Testcase(cmd) => cmd.run(),
        }
    }
}

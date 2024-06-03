mod new;
mod test;

use new::New;
use clap::Parser;
use std::error::Error;

pub trait Run {
    fn run(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cmd {
    New(New),
    Test(Test),
}

impl Cmd {
    pub fn parse() -> Cmd {
        clap::Parser::parse()
    }
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Cmd::New(cmd) => cmd.run(),
            Cmd::Test(cmd) => cmd.run(),
        }
    }
}


#[derive(Parser, Debug)]
pub struct Test {
    src_name: String,
}

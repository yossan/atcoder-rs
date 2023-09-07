mod new;
mod test;

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

///  Creates new cargo project
#[derive(Parser, Debug)]
pub struct New {
    /// project name
    #[arg(value_name = "TEXT")]
    pub name: String,
    /// files
    #[arg(value_name = "TEXT", default_values = ["a", "b", "c", "d"])]
    pub files: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct Test {
    src_name: String,
}

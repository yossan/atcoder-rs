use crate::cmd::Run;
use crate::config::{TEMPLATE, TESTCASE_DIR_NAME};

use clap::Parser;

use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;

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

impl Run for New {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let New { name, files } = self;

        // $ cargo new name
        let mut cargo_new = Command::new("cargo");
        cargo_new.arg("new").arg(name);

        // $ cargo add proconio
        let mut cargo_add = Command::new("cargo");
        cargo_add.arg("add").arg("proconio");

        make_cargo_project(name, cargo_new, cargo_add)?;

        // Remove main.rs
        fs::remove_file("src/main.rs")?;

        // Create bin folder
        fs::create_dir("src/bin")?;

        // Create source files
        for fname in files {
            fs::File::create(format!("src/bin/{fname}.rs")).unwrap();
            fs::write(format!("src/bin/{fname}.rs"), TEMPLATE).unwrap();
        }

        // Create testcase folder
        fs::create_dir(TESTCASE_DIR_NAME)?;

        Ok(())
    }
}

use crate::syscommand::{SysChild, SysCommand};

fn make_cargo_project<S, C>(
    name: &str,
    mut cargo_new: S,
    mut cargo_add: S,
) -> Result<(), Box<dyn Error>>
where
    S: SysCommand<C>,
    C: SysChild,
{
    // cargo new `name`
    cargo_new.status()?;

    // cd name
    env::set_current_dir(name).unwrap();

    // cargo add proconio
    cargo_add.status()?;

    Ok(())
}

use crate::cmd::{New, Run};

use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;

const TEMPLATE: &str = "\
use proconio::*;

fn main() {
    input! {
    }
}
";

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

        // Create template files for AtCoder
        for fname in files {
            fs::File::create(format!("src/bin/{fname}.rs")).unwrap();
            fs::write(format!("src/bin/{fname}.rs"), TEMPLATE).unwrap();
        }

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

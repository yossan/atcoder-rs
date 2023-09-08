use crate::cmd::{Run, Test};

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::syscommand::{SysChild, SysCommand};

impl Run for Test {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let Test { src_name } = self;

        // Read from testcase
        let testcase_dir = PathBuf::from(format!("testcase/{}", src_name.to_uppercase()));
        let testcase_in = testcase_dir.join("in");
        let testcase_out = testcase_dir.join("out");
        for dir_e in testcase_in.read_dir()? {
            let Ok(dir_e) = dir_e else { continue };
            let mut cargo = Command::new("cargo");
            cargo
                .arg("run")
                .arg("--bin")
                .arg(src_name)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

			let in_path = &dir_e.path();
			let Some(in_file_name) = in_path.file_name().and_then(std::ffi::OsStr::to_str) else { continue; };
			if in_path.extension().and_then(std::ffi::OsStr::to_str) != Some("txt") { continue; }
            let out_file_name = testcase_out.join(dir_e.file_name());
            let Ok(in_file) = File::open(in_path) else { continue };
            let Ok(out_file) = File::open(&out_file_name) else { 
				eprintln!("testcase/out/{:?} not found.", out_file_name);
				continue 
			};
            match cargo_run(cargo, in_file, out_file) {
                Ok(_) => {
                    println!("OK: {:?}", in_file_name);
                }
                Err(e) => {
                    println!("NG: {:?}, reason = {e}", in_file_name);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct WrongAnswer;

impl std::fmt::Display for WrongAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WrongAnswer")
    }
}

impl Error for WrongAnswer {}

fn cargo_run<P, C>(
    mut cargo_cmd: P,
    mut in_file: impl Read,
    mut out_file: impl Read,
) -> Result<C, Box<dyn Error>>
where
    P: SysCommand<C>,
    C: SysChild,
{
    // Command excution
    let mut cargo = cargo_cmd.spawn()?;

    // Write in_file to stdin
    let mut in_buf = [0u8; 1024];
    loop {
        let in_bytes_read = in_file.read(&mut in_buf)?;
        let _ = cargo.stdin_write(&in_buf[..in_bytes_read])?;
        if in_bytes_read == 0 {
            break;
        }
    }

    // Read stdout and compare it to expect
    let mut stdout_buf = [0u8; 1024];
    let mut expect_buf = [0u8; 1024];
    loop {
        let stdout_bytes_read = cargo.stdout_read(&mut stdout_buf)?;
        let expect_bytes_read = out_file.read(&mut expect_buf)?;
        if stdout_bytes_read != expect_bytes_read || stdout_buf[..stdout_bytes_read] != expect_buf[..expect_bytes_read] {
            let _ = cargo.kill();
            return Err(Box::new(WrongAnswer));
        }

        if stdout_bytes_read == 0 {
            break;
        }
    }
    Ok(cargo)
}

#[cfg(test)]
mod atcoder_test {
    use super::super::super::syscommand::syscommand_test::DummyCommand;
    use super::cargo_run;
    use std::io::Cursor;

    #[test]
    fn test_ok() {
        let input = String::from("3 34\n1\n8 13 26")
            .bytes()
            .collect::<Vec<u8>>();
        let expect = String::from("13");

        let dummy_cargo = DummyCommand {
            exit_status: 0,
            out_expect: expect.clone(),
        };

        let child = cargo_run(dummy_cargo, Cursor::new(input.clone()), Cursor::new(expect));
        assert_eq!(child.is_ok(), true);
        let stdin = child.unwrap().stdin;
        assert_eq!(stdin, input);
    }
}

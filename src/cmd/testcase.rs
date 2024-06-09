use crate::cmd::Run;
use crate::config::TESTCASE_DIR_NAME;
use crate::data::CircularBuffer;
use crate::syscommand::{SysChild, SysCommand};

use clap::Parser;

use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
/// Run testcase.
pub struct Testcase {
    /// The source file name to execute.
    src_name: String,
    /// If you want to execute only specific test cases.
    #[arg(short, long)]
    in_files: Option<Vec<String>>,
    /// If the directory containing the test cases differs from the source file name.
    #[arg(short, long)]
    dir_name: Option<String>,
}

fn check_file_existance(path: &Path) -> Result<(), Box<dyn Error>> {
    match path.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            format!("Please create `{:?}`", path),
        ))),
        Err(e) => Err(Box::new(e)),
    }
}

fn diff_position(lhs: &str, rhs: &str) -> isize {
    let (mut s1, mut s2, size) = {
        if lhs.len() >= rhs.len() {
            (lhs.chars(), rhs.chars(), lhs.len())
        } else {
            (rhs.chars(), lhs.chars(), rhs.len())
        }
    };

    let mut pos = -1;

    for i in 0..size {
        let c1 = s1.next();
        let c2 = s2.next();
        if c1 != c2 {
            pos = i as isize;
            break;
        }
    }

    pos
}

#[derive(Debug)]
struct CargoError {
    #[allow(dead_code)]
    exit_code: i32,
}

impl Display for CargoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for CargoError {}

impl Run for Testcase {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let Testcase {
            src_name,
            in_files,
            dir_name,
        } = self;
        let testcase_dir = if let Some(dir_name) = dir_name {
            PathBuf::from(format!("{}/{}", TESTCASE_DIR_NAME, dir_name))
        } else {
            PathBuf::from(format!("{}/{}", TESTCASE_DIR_NAME, src_name))
        };
        check_file_existance(&testcase_dir)?;
        let testcase_in = testcase_dir.join("in");
        check_file_existance(&testcase_in)?;
        let testcase_out = testcase_dir.join("out");
        check_file_existance(&testcase_out)?;

        // Run source programming using test cases.
        for in_entry in testcase_in.read_dir()? {
            let Ok(in_entry) = in_entry else { continue };
            let in_path = &in_entry.path();
            let Some(in_file_name_with_ext) = in_path.file_name().and_then(std::ffi::OsStr::to_str)
            else {
                continue;
            };
            if in_path.extension().and_then(std::ffi::OsStr::to_str) != Some("txt") {
                continue;
            }

            if let Some(in_files) = in_files {
                if let Some((in_file_name, _)) = in_file_name_with_ext.split_once('.') {
                    if !in_files.contains(&in_file_name.to_string()) {
                        continue;
                    }
                } else {
                    continue;
                }
            };

            let out_file_name = testcase_out.join(in_entry.file_name());
            check_file_existance(&out_file_name)?;
            let in_file = File::open(in_path)?;
            let out_file = File::open(&out_file_name)?;

            println!("testcase/{src_name}/{in_file_name_with_ext} runs. 🏃‍➡️",);

            let mut cargo = Command::new("cargo");
            cargo
                .arg("run")
                .arg("--bin")
                .arg(src_name)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            match cargo_run(cargo, BufReader::new(in_file), BufReader::new(out_file)) {
                Ok(_) => {
                    println!("OK: {:?}", in_file_name_with_ext);
                }
                Err(e) => {
                    println!("Err: {:?}, {:?}", in_file_name_with_ext, e);
                }
            }
            println!("");
        }
        Ok(())
    }
}

struct WrongAnswer {
    message: String,
}

impl std::fmt::Display for WrongAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WrongAnswer:\n{}", self.message)
    }
}

impl std::fmt::Debug for WrongAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WrongAnswer:\n{}", self.message)
    }
}

impl Error for WrongAnswer {}

fn cargo_run<P, C>(
    mut cargo_cmd: P,
    mut in_reader: impl Read,
    mut expect_reader: impl BufRead,
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
        let in_bytes_read = in_reader.read(&mut in_buf)?;
        let _ = cargo.stdin_write(&in_buf[..in_bytes_read])?;
        if in_bytes_read == 0 {
            break;
        }
    }

    // Wait for cargo to finish.
    let exit_code = cargo.exit_code()?;
    if exit_code > 0 {
        return Err(Box::new(CargoError { exit_code }));
    }

    // Read cargo's output.
    const NUM_TO_DISPLAY: usize = 5;
    type CircularBufferS = CircularBuffer<String, NUM_TO_DISPLAY>;
    let wrong_answer =
        |num_col: usize, num_row: usize, cmd_out: CircularBufferS, expect: CircularBufferS| {
            let mut message = String::new();
            message.push_str(format!("Program output(Line {}):\n", num_col).as_str());
            for line in cmd_out.iter() {
                message.push_str(format!("{:4}{}", " ", line).as_str());
            }
            message.push_str("\n");
            message.push_str(format!("{:4}{:width$}^", " ", " ", width = num_row).as_str());
            message.push_str("\n");
            message.push_str("expect:\n");

            for line in expect.iter() {
                message.push_str(format!("{:4}{}", " ", line).as_str());
            }
            message.push_str("\n");
            message.push_str(format!("{:4}{:width$}^", " ", " ", width = num_row).as_str());

            Box::new(WrongAnswer { message })
        };

    // Read the stdout of cmd byte by byte and compare it to expect
    let mut stdout_line_buf = CircularBufferS::new(NUM_TO_DISPLAY);
    let mut stdout_line = String::new();
    let mut expect_line_buf = CircularBufferS::new(NUM_TO_DISPLAY);
    let mut expect_line = String::new();
    let mut num_col = 0;
    loop {
        let stdout_read_num = cargo.stdout_read_line(&mut stdout_line)?;
        let expect_read_num = expect_reader.read_line(&mut expect_line)?;
        let diff_position = diff_position(stdout_line.trim(), expect_line.trim());

        stdout_line_buf.push(stdout_line.clone());
        expect_line_buf.push(expect_line.clone());
        stdout_line.clear();
        expect_line.clear();

        num_col += 1;

        if diff_position >= 0 {
            let _ = cargo.kill();
            return Err(wrong_answer(
                num_col,
                diff_position as usize,
                stdout_line_buf,
                expect_line_buf,
            ));
        }

        if stdout_read_num == 0 && expect_read_num == 0 {
            break;
        }
    }
    Ok(cargo)
}

#[cfg(test)]
mod atcoder_test {
    use super::super::super::syscommand::syscommand_test::DummyCommand;
    use super::cargo_run;
    use std::fs;
    use std::io::Cursor;

    #[test]
    fn test_ok() {
        let expect = String::from("a\nbc\ndef\n");
        let dummy_cargo = DummyCommand {
            exit_code: 0,
            stdout: expect.clone(),
        };
        let child = cargo_run(dummy_cargo, "dummy_input".as_bytes(), Cursor::new(expect));
        assert_eq!(child.is_ok(), true);
    }

    #[test]
    fn test_wronganswer_description() {
        let expect = String::from("abc\ndef\nghi\njklm\nopqr\ns\nt\nuvwz");
        let dummy_cargo = DummyCommand {
            exit_code: 0,
            stdout: "abc\ndef\nghi\njklm\nopqr\ns\nt\nuvw".to_string(),
        };
        let child = cargo_run(dummy_cargo, "dummy_input".as_bytes(), Cursor::new(expect));
        assert_eq!(child.is_ok(), false);
        match &child {
            Err(e) => {
                dbg!(e);
                assert_eq!(
                    e.to_string(),
                    "WrongAnswer:
Program output(Line 8):
    jklm
    opqr
    s
    t
    uvw
       ^
expect:
    jklm
    opqr
    s
    t
    uvwz
       ^"
                );
            }
            _ => {
                assert!(false);
            }
        }
    }

    fn testcase_expect(name: &str) -> String {
        let path = format!("tests/testcase/expect/{name}.txt");
        fs::read_to_string(path).unwrap()
    }
    fn testcase_pgout(name: &str) -> String {
        let path = format!("tests/testcase/program_out/{name}.txt");
        fs::read_to_string(path).unwrap()
    }

    fn run_testcase_ac(expect: String, program_out: String) {
        let dummy_cargo = DummyCommand {
            exit_code: 0,
            stdout: program_out,
        };
        let child = cargo_run(dummy_cargo, "dummy_input".as_bytes(), expect.as_bytes());
        println!("child = {:?}", child);
        assert_eq!(child.is_ok(), true);
    }

    fn run_testcase_wa(expect: String, program_out: String) {
        let dummy_cargo = DummyCommand {
            exit_code: 0,
            stdout: program_out,
        };
        let child = cargo_run(dummy_cargo, "dummy_input".as_bytes(), expect.as_bytes());
        assert_eq!(child.is_ok(), false);
    }

    #[test]
    fn testcase_num1_ac() {
        let expect = testcase_expect("num1");
        let program_out = testcase_pgout("num1_ac");
        run_testcase_ac(expect, program_out);
    }

    #[test]
    fn testcase_num1_ac_leading_wihtespace() {
        let expect = testcase_expect("num1");
        let program_out = testcase_pgout("num1_ac_leading_whitespace");
        run_testcase_ac(expect, program_out);
    }

    #[test]
    fn testcase_num1_ac_trailing_wihtespace() {
        let expect = testcase_expect("num1");
        let program_out = testcase_pgout("num1_ac_trailing_whitespace");
        run_testcase_ac(expect, program_out);
    }

    #[test]
    fn testcase_num1_wa() {
        let expect = testcase_expect("num1");
        let program_out = testcase_pgout("num1_wa");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_num1_wa_empty() {
        let expect = testcase_expect("num1");
        let program_out = testcase_pgout("num1_wa_empty");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_grid_ac() {
        let expect = testcase_expect("grid");
        let program_out = testcase_pgout("grid_ac");
        run_testcase_ac(expect, program_out);
    }

    #[test]
    fn testcase_grid_wa() {
        let expect = testcase_expect("grid");
        let program_out = testcase_pgout("grid_wa");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_grid_wa2() {
        let expect = testcase_expect("grid");
        let program_out = testcase_pgout("grid_wa2");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_grid_wa3() {
        let expect = testcase_expect("grid");
        let program_out = testcase_pgout("grid_wa3");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_num_list_ac() {
        let expect = testcase_expect("num_list");
        let program_out = testcase_pgout("num_list_ac");
        run_testcase_ac(expect, program_out);
    }

    #[test]
    fn testcase_num_list_wa_extra() {
        let expect = testcase_expect("num_list");
        let program_out = testcase_pgout("num_list_wa_extra");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_num_list_wa_lack() {
        let expect = testcase_expect("num_list");
        let program_out = testcase_pgout("num_list_wa_extra");
        run_testcase_wa(expect, program_out);
    }

    #[test]
    fn testcase_num_list_wa_value_wrong() {
        let expect = testcase_expect("num_list");
        let program_out = testcase_pgout("num_list_wa_value_wrong");
        run_testcase_wa(expect, program_out);
    }
}

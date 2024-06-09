use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::process::{Child, Command};

pub trait SysCommand<C: SysChild> {
    fn status(&mut self) -> Result<i32>;
    fn spawn(&mut self) -> Result<C>;
}

pub trait SysChild {
    fn stdin_write(&mut self, buf: &[u8]) -> Result<usize>;
    fn stdout_read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn stdout_read_line(&mut self, buf: &mut String) -> Result<usize> {
        let mut stdout_byte = [0u8; 1];
        let mut sum = 0;
        loop {
            let read_num = self.stdout_read(&mut stdout_byte)?;
            if read_num <= 0 {
                break;
            }
            sum += read_num;
            let c = stdout_byte[0] as char;
            buf.push(c);
            if c == '\n' {
                break;
            }
        }
        Ok(sum)
    }

    fn exit_code(&mut self) -> Result<i32>;

    fn kill(&mut self) -> Result<()>;
}

impl SysCommand<Child> for Command {
    fn status(&mut self) -> Result<i32> {
        match self.status() {
            Ok(exit_code) => match exit_code.code() {
                Some(code) => Ok(code),
                None => Err(Error::new(
                    ErrorKind::Other,
                    "Failed to retrieve the child process's status code.",
                )),
            },
            Err(e) => Err(e),
        }
    }

    fn spawn(&mut self) -> Result<Child> {
        match self.spawn() {
            Ok(c) => Ok(c),
            Err(e) => Err(e),
        }
    }
}

impl SysChild for Child {
    fn stdin_write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stdin.as_ref().unwrap().write(buf)
    }

    fn stdout_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let child_out = self.stdout.as_mut().unwrap();
        child_out.read(buf)
    }

    fn exit_code(&mut self) -> Result<i32> {
        match self.wait() {
            Ok(exit_code) => match exit_code.code() {
                Some(code) => Ok(code),
                None => Err(Error::new(
                    ErrorKind::Other,
                    "Failed to retrieve the child process's status code.",
                )),
            },
            Err(e) => Err(e),
        }
    }

    fn kill(&mut self) -> Result<()> {
        self.kill()
    }
}

#[cfg(test)]
pub(crate) mod syscommand_test {
    use super::{SysChild, SysCommand};
    use std::fmt::{self, Debug, Formatter};
    use std::io::prelude::*;
    use std::io::Cursor;
    use std::io::Result;

    pub trait ReadDebug: Read + Debug {}

    impl ReadDebug for Cursor<String> {}

    pub(crate) struct DummyCommand {
        pub(crate) exit_code: i32,
        pub(crate) stdout: String,
    }

    pub(crate) struct DummyChild {
        pub(crate) exit_code: i32,
        pub(crate) stdin: Vec<u8>,
        pub(crate) stdout: Box<dyn ReadDebug>,
    }

    impl Debug for DummyChild {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.debug_struct("DummyChild")
                .field("stdin", &self.stdin)
                .field("stdout", &self.stdout)
                .finish()
        }
    }

    impl SysCommand<DummyChild> for DummyCommand {
        fn status(&mut self) -> Result<i32> {
            Ok(self.exit_code)
        }

        fn spawn(&mut self) -> Result<DummyChild> {
            Ok(DummyChild {
                exit_code: self.exit_code,
                stdin: Vec::new(),
                stdout: Box::new(Cursor::new(self.stdout.clone())),
            })
        }
    }

    impl SysChild for DummyChild {
        fn stdin_write(&mut self, buf: &[u8]) -> Result<usize> {
            self.stdin.write(buf)
        }

        fn stdout_read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.stdout.read(buf)
        }

        fn exit_code(&mut self) -> Result<i32> {
            Ok(self.exit_code)
        }

        fn kill(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

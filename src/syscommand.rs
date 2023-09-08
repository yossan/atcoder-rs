use std::io::prelude::*;
use std::io::Result;
use std::process::Child;
use std::process::Command;

pub trait SysCommand<C: SysChild> {
    fn status(&mut self) -> Result<u8>;
    fn spawn(&mut self) -> Result<C>;
}

pub trait SysChild {
    fn stdin_write(&mut self, buf: &[u8]) -> Result<usize>;
    fn stdout_read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn kill(&mut self) -> Result<()>;
}

impl SysCommand<Child> for Command {
    fn status(&mut self) -> Result<u8> {
        match self.status() {
            Ok(_) => Ok(0),
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

    fn kill(&mut self) -> Result<()> {
        self.kill()
    }
}

#[cfg(test)]
pub(crate) mod syscommand_test {
    use super::{SysChild, SysCommand};
    use std::io::prelude::*;
    use std::io::Cursor;
    use std::io::Result;

    pub(crate) struct DummyCommand {
        pub(crate) exit_status: u8,
        pub(crate) out_expect: String,
    }

    pub(crate) struct DummyChild {
        pub(crate) stdin: Vec<u8>,
        pub(crate) stdout: Box<dyn Read>,
    }

    impl SysCommand<DummyChild> for DummyCommand {
        fn status(&mut self) -> Result<u8> {
            Ok(self.exit_status)
        }

        fn spawn(&mut self) -> Result<DummyChild> {
            Ok(DummyChild {
                stdin: Vec::new(),
                stdout: Box::new(Cursor::new(self.out_expect.clone())),
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

        fn kill(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

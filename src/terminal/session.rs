use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{self, BufRead, BufReader, Read};

pub enum Session {
    Stdin(BufReader<io::Stdin>),
    Pty {
        child: Box<dyn portable_pty::Child + Send + Sync>,   // exact return type
        reader: BufReader<Box<dyn Read + Send>>,
    },
}

impl Session {
    pub fn stdin() -> Self {
        Session::Stdin(BufReader::new(io::stdin()))
    }

    pub fn spawn(command: &str, args: &[String]) -> io::Result<Self> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let reader: Box<dyn Read + Send> = pair
            .master
            .try_clone_reader()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let reader = BufReader::new(reader);

        drop(pair.master);
        drop(pair.slave);

        Ok(Session::Pty { child, reader })   // child is already Box<dyn Child + Send + Sync>
    }

    pub fn reader(&mut self) -> &mut dyn BufRead {
        match self {
            Session::Stdin(ref mut r) => r,
            Session::Pty {
                child: _,
                ref mut reader,
            } => reader,
        }
    }
}
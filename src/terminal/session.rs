use portable_pty::{self, CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{self, BufRead, BufReader, Read};
use std::process::Command;
use std::sync::Arc;
use std::thread;

/// Either a spawned PTY subprocess or a simple stdin reader.
pub enum Session {
    /// Reads from standard input (original behaviour).
    Stdin(BufReader<io::Stdin>),
    /// Spawned a command under a PTY; reads from the PTY master.
    Pty {
        // Keep the child process alive for the session duration.
        child: portable_pty::Child,
        // The master side of the PTY (wrapped for buffered reading).
        reader: BufReader<Box<dyn Read + Send>>,
    },
}

impl Session {
    /// Create a session that reads from stdin.
    pub fn stdin() -> Self {
        Session::Stdin(BufReader::new(io::stdin()))
    }

    /// Spawn a command in a PTY, returning a session that reads its output.
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

        // Close slave in this process; we only need the master.
        drop(pair.slave);

        // Wrap the master in a buffered reader.
        let reader: Box<dyn Read + Send> = Box::new(pair.master);
        let reader = BufReader::new(reader);

        Ok(Session::Pty { child, reader })
    }

    /// Obtain a mutable reference to the buffered reader.
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
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::thread;

/// A terminal session – either reading from piped stdin or spawning a child
/// inside a PTY so that the child believes it has a real terminal.
pub enum Session {
    /// Sentinel is being used as a pipe filter:  `docker compose up | sentinel`
    Stdin(BufReader<io::Stdin>),

    /// Sentinel spawned the command itself:  `sentinel docker compose up --build`
    ///
    /// The child runs inside a PTY so it keeps full terminal behaviour
    /// (colour output, interactive prompts, Ctrl+C, window-size queries).
    Pty {
        child: Box<dyn portable_pty::Child + Send + Sync>,
        reader: BufReader<Box<dyn Read + Send>>,
    },
}

impl Session {
    /// Create a passthrough session that reads from the real stdin.
    pub fn stdin() -> Self {
        Session::Stdin(BufReader::new(io::stdin()))
    }

    /// Spawn `command` inside a PTY so it thinks it has a real terminal.
    ///
    /// Two background mechanisms are set up:
    ///
    /// 1. **stdin forwarding** – a thread copies bytes from the real terminal
    ///    to the PTY master write end, so Ctrl+C, Ctrl+D, and any interactive
    ///    input reach the child correctly.
    ///
    /// 2. **Terminal size** – we query the real terminal dimensions via
    ///    environment variables or fall back to 80x24.
    pub fn spawn(command: &str, args: &[String]) -> io::Result<Self> {
        let pty_system = NativePtySystem::default();

        // Use the actual terminal size so tools like docker compose format
        // their output correctly (line wrapping, progress bars, etc.).
        let size = real_terminal_size();

        let pair = pty_system
            .openpty(size)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Clone a reader for the child's output (stdout + stderr merged by PTY).
        let output_reader: Box<dyn Read + Send> = pair
            .master
            .try_clone_reader()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Take the writer so we can forward real stdin to the child.
        let mut stdin_writer: Box<dyn Write + Send> = pair
            .master
            .take_writer()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // ── Stdin forwarding thread ───────────────────────────────────────
        //
        // Copies bytes from the real terminal stdin to the PTY master write
        // end.  This makes the following work transparently:
        //   • Ctrl+C  – sends SIGINT to child (via TTY discipline)
        //   • Ctrl+D  – EOF / process exit
        //   • Ctrl+Z  – SIGTSTP (pause / resume)
        //   • Any interactive text input (e.g. `docker exec -it container bash`)
        //
        // The thread exits silently when stdin closes or the PTY closes.
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut locked = stdin.lock();
            let mut buf = [0u8; 256];
            loop {
                match locked.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if stdin_writer.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // The slave and the original master handle can be closed; the cloned
        // reader and the taken writer hold the actual file-descriptor handles.
        drop(pair.slave);
        drop(pair.master);

        Ok(Session::Pty {
            child,
            reader: BufReader::new(output_reader),
        })
    }

    /// Return a `BufRead` reference for the runtime to consume.
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

// ─────────────────────────────────────────────────────────────────────────────
// Terminal size detection
// ─────────────────────────────────────────────────────────────────────────────

/// Query the real terminal dimensions.
///
/// Checks COLUMNS / LINES environment variables first (set by most shells
/// in interactive sessions).  Falls back to 80×24 if neither is available.
///
/// This avoids adding libc as a compile-time dependency while still giving
/// the child process the correct column count for progress bars and wrapping.
fn real_terminal_size() -> PtySize {
    let cols: u16 = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&c| c > 0)
        .unwrap_or(0);

    let rows: u16 = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&r| r > 0)
        .unwrap_or(0);

    if cols > 0 && rows > 0 {
        PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }
    } else {
        // Sensible default for most developer terminals.
        PtySize {
            rows: 24,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

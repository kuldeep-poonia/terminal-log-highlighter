# Sentinel

Invisible terminal-native log highlighter. Sits between you and your tools and highlights errors, warnings, and custom patterns in real time — with zero change to how you work.

```
web-1  | Starting server on port 3000...
web-1  | Connected to database
web-1  | Error: connection refused to redis:6379        ← bright white on red + 🔔
db-1   | ready to accept connections
web-1  | Fatal: max retries exceeded                    ← bright yellow on red + 🔔
```

No daemon. No background process. No logs stored anywhere. No config required to start.

---

## How it works

Sentinel spawns your command inside a PTY (pseudo-terminal) so the command thinks it has a real terminal — colours, window size, Ctrl+C, all preserved. It reads every line of output, checks it against your patterns, and either passes it through untouched or wraps it in a highlight colour before writing it to your screen.

The whole thing is invisible. You run your tools exactly as before.

---

## Install

**One command:**

```bash
curl -fsSL https://raw.githubusercontent.com/kuldeep-poonia/terminal-log-highlighter/main/install.sh | bash
```

Restart your terminal after it finishes. Done.

**Build from source:**

```bash
git clone https://github.com/kuldeep-poonia/terminal-log-highlighter.git
cd terminal-log-highlighter
cargo build --release
sudo cp target/release/sentinel /usr/local/bin/sentinel
```

---

## Usage

### Option 1 — Invisible mode (recommended)

Add wrapper functions to your shell config once. After that, your tools work exactly as before with highlighting automatic.

```bash
# Add to ~/.zshrc or ~/.bashrc
_sentinel_run() {
    if command -v sentinel >/dev/null 2>&1; then
        sentinel "$@"
    else
        command "$@"
    fi
}

docker()  { _sentinel_run docker  "$@"; }
npm()     { _sentinel_run npm     "$@"; }
cargo()   { _sentinel_run cargo   "$@"; }
kubectl() { _sentinel_run kubectl "$@"; }
make()    { _sentinel_run make    "$@"; }
python3() { _sentinel_run python3 "$@"; }
go()      { _sentinel_run go      "$@"; }
```

```bash
source ~/.zshrc
```

Now run your tools as you always have:

```bash
docker compose up --build     # sentinel runs invisibly
npm run dev                   # sentinel runs invisibly
cargo test                    # sentinel runs invisibly
```

### Option 2 — Prefix mode

Prefix any command with `sentinel`:

```bash
sentinel docker compose up --build
sentinel npm run start
sentinel cargo build
sentinel python manage.py runserver
sentinel kubectl logs -f my-pod
```

### Option 3 — Pipe mode

```bash
docker compose up --build 2>&1 | sentinel
./my-script.sh 2>&1 | sentinel
```

> Pipe mode may suppress colours from the original command because it detects it is not writing to a real terminal. Option 1 or 2 is preferred.

### Bypass sentinel for a single command

```bash
command docker compose up --build    # skips sentinel, calls real docker
command npm run dev
```

---

## Highlight colours

| Severity | Appearance | Bell |
|----------|------------|------|
| `critical` | Bold bright yellow on red background | 🔔 Yes |
| `error` | Bold bright white on red background | 🔔 Yes |
| `warn` | Bold bright yellow | No |
| `info` | Bold bright cyan | No |

The bell is a single ASCII `BEL` byte written to stdout. No audio files. Your terminal emulator plays its own alert sound. Mute it in your terminal settings if you prefer silent-only.

---

## Configuration

Sentinel works with no config file. Built-in patterns cover the most common signals.

To add your own, create a `.sentinel.toml`. Sentinel searches in this order:

1. `$SENTINEL_CONFIG` — explicit path via environment variable
2. `./.sentinel.toml` — project-local, in the current directory
3. `~/.sentinel.toml` — user-wide, in your home directory
4. Built-in defaults

**Example `.sentinel.toml`:**

```toml
[filters.error]
pattern  = "error"
severity = "error"

[filters.warn]
pattern  = "warn"
severity = "warn"

[filters.ready]
pattern  = "ready"
severity = "info"

# Your own custom rules
[filters.my_alert]
pattern  = "PAYMENT_FAILED"
severity = "critical"

[filters.slow_query]
pattern  = "slow query"
severity = "warn"
```

**Severity values:** `critical` · `error` · `warn` · `info`

Patterns match as **case-insensitive substrings** anywhere in the line. `"error"` matches `Error`, `ERROR`, `BuildError`, `connection error:`, etc.

---

## Built-in patterns

**Critical:** `panic`, `fatal`, `out of memory`, `oom`, `segfault`, `killed`, `timeout`, `deadlock`

**Error:** `error`, `exception`, `traceback`, `failed`, `failure`, `crash`, `denied`, `refused`, `not found`, `unhandled`, `uncaught`

**Warn:** `warn`, `deprecated`, `retrying`, `retry`, `slow`

**Info:** `success`, `ready`, `listening`, `connected`, `started`, `built`

---

## Quick test

Run this immediately after installing:

```bash
echo -e "server starting...\nerror: port 3000 in use\nwarning: retrying on 3001\nserver ready" | sentinel
```

---

## Features

- Zero latency — lines are highlighted the moment they are written
- Zero config required — works out of the box with sensible defaults
- Invisible by design — use your tools unchanged after one-time setup
- No storage — sentinel never stores, logs, or forwards your output
- PTY-based — preserves colour output, interactive prompts, correct window size
- Ctrl+C / Ctrl+D work correctly — stdin is forwarded to the child process
- Audible alerts — terminal bell on error and critical lines, no audio files
- Custom patterns — add your own keywords and severity levels
- Regex support — build with `--features regex` for full regular expression patterns

---

## Uninstall

```bash
sudo rm /usr/local/bin/sentinel
# Remove the wrapper block from ~/.zshrc or ~/.bashrc
source ~/.zshrc
```

---

## License

MIT
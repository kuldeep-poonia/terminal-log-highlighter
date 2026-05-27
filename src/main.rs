mod runtime;
mod matcher;

fn main() {
    // Invisible runtime: no output on failure.
    // In a later phase, the environment variable SENTINEL_DEBUG will enable
    // silent internal diagnostics without breaking the invisible promise.
    if let Err(e) = runtime::run() {
        std::process::exit(e.raw_os_error().unwrap_or(1));
    }
}
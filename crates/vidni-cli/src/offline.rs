//! `--offline`: the command runs again in a namespace with no network, so
//! that a build which reaches for the network fails instead of passing by
//! luck (TODO.md, P3-VIDNI-06). Linux only: it needs `unshare`, from
//! util-linux, and a kernel that lets a user make namespaces.

use std::ffi::OsString;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode, Stdio};

/// Set in a command that `--offline` started.
const MARK: &str = "VIDNI_OFFLINE";

/// Whether this process runs with no network, started by `--offline`.
pub fn inside() -> bool {
    std::env::var_os(MARK).is_some_and(|v| !v.is_empty())
}

/// The interfaces this process has a route through, other than loopback,
/// as the kernel lists the routes of its network namespace. A namespace
/// made for `--offline` has none. It may still list interfaces: where
/// tunnel modules are loaded, every new namespace gets their idle devices.
pub fn routes() -> Result<Vec<String>, String> {
    let read = |file: &str| std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"));
    let mut through: Vec<String> = Vec::new();
    // IPv4: a header, then a route on each line, its interface first.
    let v4 = read("/proc/self/net/route")?;
    through.extend(
        v4.lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter(|name| *name != "Iface")
            .map(str::to_string),
    );
    // IPv6: a route on each line, its interface last.
    let v6 = read("/proc/self/net/ipv6_route")?;
    through.extend(
        v6.lines()
            .filter_map(|line| line.split_whitespace().next_back())
            .map(str::to_string),
    );
    through.retain(|name| name != "lo");
    through.sort();
    through.dedup();
    Ok(through)
}

/// Checks that a process marked as running with no network has none, so
/// that the mark can't claim what isn't so.
pub fn verify() -> Result<(), String> {
    match routes()?.as_slice() {
        [] => Ok(()),
        through => Err(format!(
            "{MARK} is set, but this process has a route to the network (through {}); `--offline` sets {MARK} itself, in a namespace with none",
            through.join(", ")
        )),
    }
}

/// The command that runs another with no network, checked to work here:
/// `unshare` with a network namespace that holds nothing, in a user
/// namespace that keeps the user's own ID. (Mapped to root instead, as
/// `unshare -r` does, Chromium would refuse to start.)
pub fn wrapper() -> Result<Vec<OsString>, String> {
    // A process's folder in /proc belongs to the user it runs as.
    let me = std::fs::metadata("/proc/self").map_err(|e| format!("/proc/self: {e}"))?;
    let wrapper: Vec<OsString> = vec![
        "unshare".into(),
        "--user".into(),
        format!("--map-user={}", me.uid()).into(),
        format!("--map-group={}", me.gid()).into(),
        "--net".into(),
        "--".into(),
    ];
    let probe = Command::new(&wrapper[0])
        .args(&wrapper[1..])
        .arg("true")
        .stdin(Stdio::null())
        .output();
    match probe {
        Ok(out) if out.status.success() => Ok(wrapper),
        Ok(out) => Err(format!(
            "--offline can't make a namespace with no network on this system: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => Err(format!("--offline needs unshare, from util-linux: {e}")),
    }
}

/// The environment mark for a command inside [`wrapper`].
pub fn mark() -> (&'static str, &'static str) {
    (MARK, "1")
}

/// Becomes this very command again, with no network and without
/// `--offline`. This process is replaced, so that signals and the exit
/// code are the command's own; it returns only when that can't be done.
pub fn again() -> ExitCode {
    let why = match (wrapper(), crate::own_exe()) {
        (Err(e), _) => e,
        (_, Err(e)) => format!("vidni's own file: {e}"),
        (Ok(wrapper), Ok(exe)) => {
            let (name, value) = mark();
            let error = Command::new(&wrapper[0])
                .args(&wrapper[1..])
                .arg(exe)
                .args(std::env::args_os().skip(1).filter(|a| a != "--offline"))
                .env(name, value)
                .exec();
            format!("unshare: {error}")
        }
    };
    tell!("vidni: {why}");
    ExitCode::from(2)
}

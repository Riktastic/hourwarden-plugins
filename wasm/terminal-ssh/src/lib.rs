//! SSH sessions in terminals: the server from the title of Windows Terminal,
//! PuTTY, Konsole and other terminals, so time on a customer's server goes to
//! that customer. It only reads the title of the window in front; nothing
//! leaves the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `terminal-ssh`
//! plugin. When the title says nothing (a custom prompt, a tab title), it asks
//! the app for the command lines of the `ssh` processes started from that
//! terminal (capability `process.ssh_commands`); one server among them names
//! the session, several (other tabs) do not.

// Single-threaded wasm: the settings statics are written once in on_init.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

/// Terminals whose window titles are read, with or without `.exe`.
const DEFAULT_APPS: &[&str] = &[
    "windowsterminal.exe", "openconsole.exe", "conhost.exe", "putty.exe", "kitty.exe", "mobaxterm.exe", "mremoteng.exe",
    "securecrt.exe", "tabby.exe", "termius.exe", "wezterm-gui.exe", "alacritty.exe", "royalts.exe", "konsole", "yakuake",
    "gnome-terminal-server", "kgx", "ptyxis", "tilix", "terminator", "xfce4-terminal", "xterm", "foot", "ghostty",
];

/// `ssh` options that take a value: the value after them is not the host.
const OPTIONS_WITH_VALUE: &[&str] =
    &["-b", "-c", "-D", "-E", "-e", "-F", "-I", "-i", "-J", "-L", "-l", "-m", "-O", "-o", "-p", "-Q", "-R", "-S", "-W", "-w"];

/// App words at the end of a terminal title, after " - ", " | " or " — ".
const SUFFIX_APPS: &[&str] = &[
    "PuTTY", "KiTTY", "SecureCRT", "Tabby", "Termius", "MobaXterm", "mRemoteNG", "Konsole", "Yakuake", "Tilix",
    "Terminator", "Terminal",
];

static mut APPS: Vec<String> = Vec::new();
static mut LOCAL: Vec<String> = Vec::new();
/// The latest tick time (ms), and the last process lookup: window, time, server.
static mut NOW: i64 = 0;
static mut PROC: Option<(i64, i64, Option<String>)> = None;
/// Process lookups take a moment: once per window every 10 seconds.
const PROC_EVERY_MS: i64 = 10_000;

fn app_name(exe: &str) -> &str {
    exe.strip_suffix(".exe").unwrap_or(exe)
}

fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'-'
}

/// Matches `^[A-Za-z0-9]([A-Za-z0-9.-]*[A-Za-z0-9])?$`.
fn is_valid_host(name: &str) -> bool {
    let b = name.as_bytes();
    if b.is_empty() || !b[0].is_ascii_alphanumeric() {
        return false;
    }
    if b.len() > 1 && !b[b.len() - 1].is_ascii_alphanumeric() {
        return false;
    }
    b.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
}

/// A real remote server: a valid host name that is not this computer, one of
/// the user's own machines, or localhost.
fn is_remote(name: &str, local: &[String]) -> bool {
    if !is_valid_host(name) {
        return false;
    }
    let short = name.split('.').next().unwrap_or(name);
    for l in local {
        if !l.is_empty() && (name.eq_ignore_ascii_case(l) || short.eq_ignore_ascii_case(l)) {
            return false;
        }
    }
    name != "localhost" && name != "127.0.0.1"
}

/// Drops a trailing " - PuTTY", " — SecureCRT" and similar app tails.
fn strip_app_suffix(title: &str) -> String {
    let seps = [" - ", " | ", " \u{2014} "];
    let mut cut = title.len();
    for sep in seps {
        let mut from = 0;
        while let Some(rel) = title[from..].find(sep) {
            let pos = from + rel;
            let after = &title[pos + sep.len()..];
            let is_app = SUFFIX_APPS
                .iter()
                .any(|w| after.starts_with(w) && after[w.len()..].chars().next().map_or(true, |c| !c.is_alphanumeric()));
            if is_app {
                cut = cut.min(pos);
                break;
            }
            from = pos + sep.len();
        }
    }
    title[..cut].trim().to_string()
}

/// Drops a leading "Administrator: " / "Administrateur: " / "Beheerder: ".
fn strip_admin_prefix(t: &str) -> &str {
    for p in ["Administrator:", "Administrateur:", "Beheerder:"] {
        if let Some(rest) = t.strip_prefix(p) {
            return rest.trim_start();
        }
    }
    t
}

/// Removes a trailing `:1234` port.
fn strip_port(s: &str) -> &str {
    if let Some(k) = s.rfind(':') {
        if k + 1 < s.len() && s[k + 1..].bytes().all(|c| c.is_ascii_digit()) {
            return &s[..k];
        }
    }
    s
}

/// The arguments after an `ssh`/`ssh.exe` token, if the title holds one.
fn find_ssh_args(t: &str) -> Option<&str> {
    let bytes = t.as_bytes();
    let mut i = 0;
    while let Some(rel) = t[i..].find("ssh") {
        let pos = i + rel;
        let before_ok = pos == 0 || matches!(bytes[pos - 1], b' ' | b'\t' | b'\\' | b'/');
        let mut end = pos + 3;
        if t[end..].starts_with(".exe") {
            end += 4;
        }
        let after = &t[end..];
        if before_ok && after.starts_with(char::is_whitespace) {
            return Some(after.trim_start());
        }
        i = pos + 3;
    }
    None
}

/// The host in an `ssh` command line's arguments.
fn host_from_ssh_args(args: &str, local: &[String]) -> Option<String> {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        if OPTIONS_WITH_VALUE.contains(&tok) {
            i += 2;
            continue;
        }
        if tok.starts_with('-') {
            i += 1;
            continue;
        }
        let name = tok.trim_matches(|c| c == '"' || c == '\'');
        let name = name.strip_prefix("ssh://").unwrap_or(name);
        let name = match name.find('@') {
            Some(k) => &name[k + 1..],
            None => name,
        };
        let name = strip_port(name);
        return is_remote(name, local).then(|| name.to_lowercase());
    }
    None
}

/// A shell prompt's host: "rik@web01: ~/app", "[admin@app01 ~]$".
fn host_from_prompt(t: &str, local: &[String]) -> Option<String> {
    let s = t.strip_prefix('[').unwrap_or(t);
    let at = s.find('@')?;
    let user = &s[..at];
    if user.is_empty() || !user.bytes().all(is_word_byte) {
        return None;
    }
    let rest = &s[at + 1..];
    let hend = rest.find(|c: char| !(c.is_ascii() && is_word_byte(c as u8))).unwrap_or(rest.len());
    let host = &rest[..hend];
    let after_ok = rest[hend..].chars().next().map_or(true, |c| matches!(c, ':' | ' ' | ']' | '\t'));
    (after_ok && is_remote(host, local)).then(|| host.to_lowercase())
}

/// A bare host title, as PuTTY and friends show (e.g. "web01.contoso.nl").
fn host_bare(t: &str, local: &[String]) -> Option<String> {
    if t.is_empty() || !t.bytes().all(is_word_byte) || !t.contains('.') {
        return None;
    }
    is_remote(t, local).then(|| t.to_lowercase())
}

/// The SSH server a terminal title points at, if any.
fn get_ssh_host(title: &str, local: &[String]) -> Option<String> {
    if title.is_empty() {
        return None;
    }
    let stripped = strip_app_suffix(title);
    let t = strip_admin_prefix(&stripped);
    if let Some(args) = find_ssh_args(t) {
        if let Some(h) = host_from_ssh_args(args, local) {
            return Some(h);
        }
    }
    host_from_prompt(t, local).or_else(|| host_bare(t, local))
}

/// The server in one `ssh` command line (`/usr/bin/ssh -p 22 admin@web01`).
fn host_from_command(line: &str, local: &[String]) -> Option<String> {
    let line = line.replace('"', "");
    host_from_ssh_args(find_ssh_args(&line)?, local)
}

/// The one server among the `ssh` commands started from terminal `pid`, if
/// there is exactly one.
fn host_from_processes(pid: u64, local: &[String]) -> Option<String> {
    let result = hw::capability("process.ssh_commands", &json!({ "pid": pid }))?;
    let mut hosts: Vec<String> = Vec::new();
    for line in result["commands"].as_array()?.iter().filter_map(Value::as_str) {
        if let Some(h) = host_from_command(line, local) {
            if !hosts.contains(&h) {
                hosts.push(h);
            }
        }
    }
    (hosts.len() == 1).then(|| hosts.remove(0))
}

fn string_list(v: &Value, default: &[&str]) -> Vec<String> {
    match v.as_array() {
        Some(a) => a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        None => default.iter().map(|s| s.to_string()).collect(),
    }
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Watching terminal titles for SSH sessions");
}

fn configure(settings: &Value) {
    let apps = string_list(&settings["apps"], DEFAULT_APPS);
    let mut local = string_list(&settings["local_hosts"], &[]);
    // The app can name this computer so its own prompt is not a remote server.
    if let Some(h) = hw::capability("hostname", &Value::Null).and_then(|v| v.as_str().map(str::to_string)) {
        local.push(h);
    }
    unsafe {
        APPS = apps;
        LOCAL = local;
    }
}

fn on_event(event: Value) {
    match event["kind"].as_str() {
        Some("tick") => unsafe { NOW = event["now"].as_i64().unwrap_or(0) },
        Some("settings") => configure(&event["settings"]),
        Some("foreground") => on_foreground(&event),
        _ => {}
    }
}

fn on_foreground(event: &Value) {
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let apps = unsafe { &APPS };
    if !apps.iter().any(|a| app_name(&a.to_lowercase()) == app_name(&exe)) {
        return;
    }
    let title = event["title"].as_str().unwrap_or("");
    let local = unsafe { &LOCAL };
    let mut server = get_ssh_host(title, local);
    if server.is_none() {
        if let (Some(pid), Some(hwnd)) = (event["pid"].as_u64().filter(|p| *p > 0), event["hwnd"].as_i64()) {
            let now = unsafe { NOW };
            let fresh = unsafe { &PROC }.as_ref().is_some_and(|(h, at, _)| *h == hwnd && now - at < PROC_EVERY_MS);
            if !fresh {
                unsafe { PROC = Some((hwnd, now, host_from_processes(pid, local))) };
            }
            server = unsafe { &PROC }.as_ref().and_then(|(_, _, s)| s.clone());
        }
    }
    if let Some(server) = server {
        hw::emit(&json!({
            "method": "detail",
            "params": { "hwnd": event["hwnd"], "exe": exe, "remote_server": server, "document": server }
        }));
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    fn host(title: &str) -> Option<String> {
        get_ssh_host(title, &["mylaptop".to_string()])
    }

    #[test]
    fn ssh_command() {
        assert_eq!(host("ssh -p 2222 -i key admin@web01.contoso.nl uptime"), Some("web01.contoso.nl".into()));
        assert_eq!(host("ssh ssh://deploy@db01.example.com:22"), Some("db01.example.com".into()));
    }

    #[test]
    fn shell_prompt() {
        assert_eq!(host("rik@web01.contoso.nl: ~/app"), Some("web01.contoso.nl".into()));
        assert_eq!(host("[admin@app01 ~]$"), Some("app01".into()));
    }

    #[test]
    fn bare_and_suffix() {
        assert_eq!(host("web01.contoso.nl - PuTTY"), Some("web01.contoso.nl".into()));
        assert_eq!(host("db01.example.com — SecureCRT"), Some("db01.example.com".into()));
    }

    #[test]
    fn ssh_process_command_lines() {
        let local = ["mylaptop".to_string()];
        assert_eq!(host_from_command("/usr/bin/ssh -p 2222 admin@web01.contoso.nl", &local), Some("web01.contoso.nl".into()));
        assert_eq!(host_from_command(r#""C:\Windows\System32\OpenSSH\ssh.exe" -i key deploy@db01"#, &local), Some("db01".into()));
        assert_eq!(host_from_command("ssh mylaptop", &local), None);
    }

    #[test]
    fn not_a_remote_host() {
        assert_eq!(host("localhost: ~"), None);
        assert_eq!(host("rik@mylaptop: ~"), None);
        assert_eq!(host("just a title"), None);
    }
}

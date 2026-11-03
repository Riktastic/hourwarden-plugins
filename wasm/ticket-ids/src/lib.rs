//! Ticket IDs in window titles: Jira (`CON-123`), TopDesk (`I 2410 123`),
//! ServiceNow (`INC0012345`) and your own patterns, offered in the top bar as
//! the task. It only reads the title of the window in front; it never decides
//! the customer and nothing leaves the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `ticket-ids`
//! plugin. Patterns are regular expressions in the `regex-lite` syntax, which
//! covers the defaults and the usual `\b`, `\d`, classes and repeats;
//! look-arounds are not supported and report the plugin as degraded.

// Single-threaded wasm: the configuration is replaced on init and settings.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};
use regex_lite::Regex;

/// Programs whose window title may hold a ticket ID, with or without `.exe`.
const DEFAULT_APPS: &[&str] =
    &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "outlook.exe", "olk.exe", "ms-teams.exe", "chromium", "teams-for-linux"];

const DEFAULT_PATTERNS: &[&str] = &[
    r"\b[A-Z][A-Z0-9]{1,9}-\d{1,7}\b",               // Jira, YouTrack, Linear: CON-123
    r"\b[IMWC] ?\d{4}[ -]\d{3,6}\b",                 // TopDesk: I 2410 123, I2410-123
    r"\b(?:INC|RITM|REQ|CHG|PRB|SCTASK|TASK)\d{7}\b", // ServiceNow: INC0012345
];

/// Things that look like ticket IDs but are not (matched ignoring case).
const DEFAULT_IGNORE: &str =
    "^(UTF|ISO|SHA|MD|COVID|WIN|TLS|SSL|IPV|HTTP|RFC|CVE|GPT|AES|RSA|USB|HDMI|PCI|DDR|LTE|IEEE|ASCII|ES|EN|NL|DE)-";

/// Mail reply and forward prefixes in several languages.
const REPLY_PREFIX: &str = r"(?i)^\s*((RE|FW|FWD|AW|WG|TR|ANTW|DOORST)\s*:\s*)+";

struct Config {
    apps: Vec<String>,
    patterns: Vec<Regex>,
    ignore: Option<Regex>,
    separator: Regex,
    reply: Regex,
}

static mut CONFIG: Option<Config> = None;

fn app_name(exe: &str) -> String {
    let exe = exe.to_lowercase();
    exe.strip_suffix(".exe").unwrap_or(&exe).to_string()
}

fn string_list(v: &Value, default: &[&str]) -> Vec<String> {
    match v.as_array() {
        Some(a) => a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        None => default.iter().map(|s| s.to_string()).collect(),
    }
}

/// Builds and stores the configuration; returns the patterns that could not
/// be compiled, so they can be reported.
fn configure(settings: &Value) -> Vec<String> {
    let (config, broken) = build(settings);
    unsafe { CONFIG = Some(config) };
    broken
}

/// The configuration from the plugin settings, and the patterns that could
/// not be compiled.
fn build(settings: &Value) -> (Config, Vec<String>) {
    let mut broken = Vec::new();
    let patterns = string_list(&settings["patterns"], DEFAULT_PATTERNS)
        .into_iter()
        .filter_map(|p| match Regex::new(&p) {
            Ok(r) => Some(r),
            Err(_) => {
                broken.push(p);
                None
            }
        })
        .collect();
    let ignore_text = settings["ignore"].as_str().unwrap_or(DEFAULT_IGNORE);
    let ignore = if ignore_text.trim().is_empty() {
        None
    } else {
        match Regex::new(&format!("(?i){ignore_text}")) {
            Ok(r) => Some(r),
            Err(_) => {
                broken.push(ignore_text.to_string());
                None
            }
        }
    };
    let config = Config {
        apps: string_list(&settings["apps"], DEFAULT_APPS).iter().map(|a| app_name(a)).collect(),
        patterns,
        ignore,
        separator: Regex::new(r"\s[-|]\s").expect("valid separator"),
        reply: Regex::new(REPLY_PREFIX).expect("valid reply prefix"),
    };
    (config, broken)
}

fn report(broken: &[String]) {
    if broken.is_empty() {
        hw::status("running", "Looking for ticket IDs in window titles");
    } else {
        hw::status("degraded", &format!("A pattern does not work: {}", broken.join(", ")));
    }
}

fn trimmable(c: char) -> bool {
    matches!(c, ' ' | '[' | ']' | '(' | ')' | ':' | '#' | '-' | '|')
}

/// The first ticket ID in a title and the words around it as the task:
/// "[CON-123] Login fails - Jira - Microsoft Edge" gives CON-123, "Login fails".
fn ticket_hint(config: &Config, title: &str) -> Option<(String, String)> {
    if title.is_empty() {
        return None;
    }
    let segments: Vec<&str> = config.separator.split(title).collect();
    for pattern in &config.patterns {
        for (i, segment) in segments.iter().enumerate() {
            for m in pattern.find_iter(segment) {
                let id = m.as_str().split_whitespace().collect::<Vec<_>>().join(" ");
                if config.ignore.as_ref().is_some_and(|r| r.is_match(&id)) {
                    continue;
                }
                let rest = format!("{}{}", &segment[..m.start()], &segment[m.end()..]);
                let mut words = config.reply.replace(&rest, "").trim_matches(trimmable).trim().to_string();
                // TopDesk puts the number in its own part: "I 2410 123 - Printer broken - TOPdesk".
                if words.is_empty() && i + 2 < segments.len() {
                    words = segments[i + 1].trim().to_string();
                }
                return Some((id, words));
            }
        }
    }
    None
}

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 2) };
    let broken = configure(&data["settings"]);
    report(&broken);
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 2000;

fn on_event(event: Value) {
    if event["kind"] == "settings" {
        unsafe { POLL_MS = hw::poll_ms(&event["settings"], 2) };
        let broken = configure(&event["settings"]);
        report(&broken);
    } else if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        on_foreground(&fg);
    }
}

fn on_foreground(event: &Value) {
    let Some(config) = (unsafe { CONFIG.as_ref() }) else { return };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    if !config.apps.contains(&app_name(&exe)) {
        return;
    }
    if let Some((id, words)) = ticket_hint(config, event["title"].as_str().unwrap_or("")) {
        hw::emit(&json!({
            "method": "detail",
            "params": { "hwnd": event["hwnd"], "exe": exe, "ticket_id": id, "ticket_title": words }
        }));
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(title: &str) -> Option<(String, String)> {
        ticket_hint(&build(&json!({})).0, title)
    }

    #[test]
    fn jira() {
        assert_eq!(hint("[CON-123] Login fails - Jira - Microsoft Edge"), Some(("CON-123".into(), "Login fails".into())));
        assert_eq!(hint("RE: FAB-7 Printer - Outlook"), Some(("FAB-7".into(), "Printer".into())));
    }

    #[test]
    fn topdesk_number_in_own_part() {
        assert_eq!(hint("I 2410 123 - Printer broken - TOPdesk"), Some(("I 2410 123".into(), "Printer broken".into())));
    }

    #[test]
    fn servicenow() {
        assert_eq!(hint("INC0012345 | Mail down - ServiceNow"), Some(("INC0012345".into(), "Mail down".into())));
    }

    #[test]
    fn ignored_lookalikes() {
        assert_eq!(hint("UTF-8 encoding guide - Edge"), None);
        assert_eq!(hint("no ticket here"), None);
    }

    #[test]
    fn broken_pattern_is_reported() {
        assert_eq!(build(&json!({ "patterns": ["(?<=x)y", r"\bOK-\d+\b"] })).1, vec!["(?<=x)y".to_string()]);
    }
}

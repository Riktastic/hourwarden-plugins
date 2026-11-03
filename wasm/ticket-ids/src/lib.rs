//! Ticket IDs in window titles (Jira `CON-123`, and project keys like it),
//! offered in the top bar as the task. It only looks at the title of the
//! window in front; nothing leaves the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `ticket-ids`
//! plugin. It carries the common Jira-style pattern; the remaining patterns
//! (TopDesk, ServiceNow and user patterns) come with the other ports.

use hourwarden_plugin as hw;
use hw::{json, Value};

/// Programs whose window title may hold a ticket ID.
const APPS: &[&str] = &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "outlook.exe", "olk.exe", "ms-teams.exe", "chromium", "teams-for-linux"];

/// Finds a `KEY-123` style ticket ID and the words around it, if any.
/// A key is 2–10 upper-case letters/digits starting with a letter, then a
/// hyphen and 1–7 digits: `CON-123`, `FAB-7`, `AB12-9`.
fn parse_ticket(title: &str) -> Option<(String, String)> {
    let bytes = title.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // A key starts at a word boundary with an upper-case letter.
        let boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if boundary && bytes[i].is_ascii_uppercase() {
            let start = i;
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_uppercase() || bytes[j].is_ascii_digit()) && j - start <= 10 {
                j += 1;
            }
            // 2..=10 chars of key, then '-' and 1..=7 digits.
            if j - start >= 2 && j - start <= 10 && j < bytes.len() && bytes[j] == b'-' {
                let digits_start = j + 1;
                let mut k = digits_start;
                while k < bytes.len() && bytes[k].is_ascii_digit() && k - digits_start < 7 {
                    k += 1;
                }
                let ends = k == bytes.len() || !bytes[k].is_ascii_alphanumeric();
                if k > digits_start && ends {
                    let id = title[start..k].to_string();
                    let title_words = surrounding_words(title, start, k);
                    return Some((id, title_words));
                }
            }
        }
        i += 1;
    }
    None
}

/// The description near the ticket ID: the title with the ID and the common
/// " - App" tail removed, trimmed of brackets and separators.
fn surrounding_words(title: &str, start: usize, end: usize) -> String {
    let without_id = format!("{}{}", &title[..start], &title[end..]);
    let first = without_id.split(" - ").find(|part| !part.trim_matches(trimmable).trim().is_empty());
    first.unwrap_or("").trim_matches(trimmable).trim().to_string()
}

fn trimmable(c: char) -> bool {
    matches!(c, '[' | ']' | '(' | ')' | ':' | '-' | ' ' | '|')
}

fn on_init(_data: Value) {
    hw::status("running", "Looking for ticket IDs in window titles");
}

fn on_event(event: Value) {
    if event["kind"] != "foreground" {
        return;
    }
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    if !APPS.iter().any(|a| a.trim_end_matches(".exe") == exe.trim_end_matches(".exe")) {
        return;
    }
    let title = event["title"].as_str().unwrap_or("");
    if let Some((id, words)) = parse_ticket(title) {
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

    #[test]
    fn jira_keys() {
        assert_eq!(parse_ticket("[CON-123] Login fails - Jira - Microsoft Edge"), Some(("CON-123".into(), "Login fails".into())));
        assert_eq!(parse_ticket("Login fails (FAB-7) - Jira"), Some(("FAB-7".into(), "Login fails".into())));
        assert_eq!(parse_ticket("no ticket here"), None);
        assert_eq!(parse_ticket("lowercase con-1 ignored"), None);
    }
}

//! Teams meetings: a meeting runs while Teams holds the microphone, named
//! after the Teams window that is not one of its main sections.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `teams-meetings` plugin. The app reads the microphone state and Teams'
//! window titles (capability `teams.state`); choosing the meeting is done here.

// Single-threaded wasm: the state lives in a static between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};
use regex_lite::Regex;

/// Titles of the main Teams window sections; anything else while the
/// microphone is on is the meeting.
const HUB: &str = r"(?i)^(Chat|Activity|Calendar|Teams|Calls|Files|Apps|OneDrive|Assignments|Communities|Copilot|Microsoft Teams)\b";
const TEAMS_TAIL: &str = r"(?i)\s*\|\s*Microsoft Teams.*$";

struct State {
    poll_ms: i64,
    count_any_call: bool,
    polled_at: Option<i64>,
    in_meeting: bool,
    patterns: Option<(Regex, Regex)>,
}

static mut STATE: State = State { poll_ms: 5000, count_any_call: true, polled_at: None, in_meeting: false, patterns: None };

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.poll_ms = hw::setting_i64(&settings["poll_seconds"]).unwrap_or(5).clamp(1, 60) * 1000;
    s.count_any_call = settings["count_any_call"].as_bool().unwrap_or(true);
}

/// The meeting's name: the first window title that is not a main section.
fn meeting_title(titles: &[&str], hub: &Regex, tail: &Regex) -> Option<String> {
    titles.iter().map(|t| tail.replace(t, "").trim().to_string()).find(|t| !t.is_empty() && !hub.is_match(t))
}

fn on_tick(now: i64) {
    let s = unsafe { &mut STATE };
    if s.polled_at.is_some_and(|at| now - at < s.poll_ms) {
        return;
    }
    s.polled_at = Some(now);
    let Some(answer) = hw::capability("teams.state", &Value::Null) else {
        hw::status("degraded", "Teams could not be checked");
        return;
    };
    match answer["mic"].as_bool() {
        None => hw::status("degraded", "The microphone privacy registry key was not found"),
        Some(true) => {
            let (hub, tail) = s.patterns.get_or_insert_with(|| (Regex::new(HUB).unwrap(), Regex::new(TEAMS_TAIL).unwrap()));
            let titles: Vec<&str> = answer["titles"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
            let title = meeting_title(&titles, hub, tail).or_else(|| s.count_any_call.then(|| "Teams call".to_string()));
            if let Some(title) = title {
                // Repeated while it lasts, so the app knows the meeting is still on.
                hw::emit(&json!({ "method": "meeting", "params": { "active": true, "title": title } }));
                if !s.in_meeting {
                    hw::log(&format!("Meeting started: {title}"));
                }
                s.in_meeting = true;
            }
        }
        Some(false) => {
            if s.in_meeting {
                hw::emit(&json!({ "method": "meeting", "params": { "active": false } }));
                hw::log("Meeting ended");
                s.in_meeting = false;
            }
        }
    }
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Watching the microphone");
}

fn on_event(event: Value) {
    match event["kind"].as_str() {
        Some("tick") => on_tick(event["now"].as_i64().unwrap_or(0)),
        Some("settings") => configure(&event["settings"]),
        _ => {}
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_meeting_is_the_window_that_is_not_a_section() {
        let (hub, tail) = (Regex::new(HUB).unwrap(), Regex::new(TEAMS_TAIL).unwrap());
        let titles = ["Chat | Microsoft Teams", "Weekly sync Contoso | Microsoft Teams", "Calendar | Microsoft Teams"];
        assert_eq!(meeting_title(&titles, &hub, &tail), Some("Weekly sync Contoso".into()));
        assert_eq!(meeting_title(&["Chat | Microsoft Teams", "Microsoft Teams"], &hub, &tail), None);
    }
}

//! Outlook calendar: your appointments of the last days and today, so the
//! daily review can add meetings you attended away from the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `outlook-calendar` plugin. The app reads the appointments (capability
//! `outlook.calendar`): time, subject, location and attendees, never the body.
//! Which ones are time spent is decided here.

// Single-threaded wasm: the state lives in a static between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

/// Outlook's BusyStatus and MeetingStatus values used below.
const OL_FREE: i64 = 0;
const OL_MEETING_CANCELED: i64 = 5;
const OL_MEETING_RECEIVED_AND_CANCELED: i64 = 7;

struct State {
    days_back: i64,
    refresh_ms: i64,
    skip_free: bool,
    max_attendees: i64,
    ignore: Vec<String>,
    read_at: Option<i64>,
}

static mut STATE: State = State { days_back: 7, refresh_ms: 15 * 60_000, skip_free: true, max_attendees: 50, ignore: Vec::new(), read_at: None };

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.days_back = settings["days_back"].as_i64().unwrap_or(7).clamp(0, 60);
    s.refresh_ms = settings["refresh_minutes"].as_i64().unwrap_or(15).clamp(1, 1440) * 60_000;
    s.skip_free = settings["skip_free"].as_bool().unwrap_or(true);
    s.max_attendees = settings["max_attendees"].as_i64().unwrap_or(50).clamp(1, 500);
    s.ignore = hw::string_list(&settings["ignore_domains"], &[]);
    // Read again with the new settings.
    s.read_at = None;
}

/// One appointment as the app wants it, or `None` for what is not time
/// spent: all-day items, free time (when skipped) and cancelled meetings.
fn convert(item: &Value, skip_free: bool, ignore: &[String]) -> Option<Value> {
    if item["all_day"] == true {
        return None;
    }
    if skip_free && item["busy_status"].as_i64() == Some(OL_FREE) {
        return None;
    }
    if matches!(item["meeting_status"].as_i64(), Some(OL_MEETING_CANCELED | OL_MEETING_RECEIVED_AND_CANCELED)) {
        return None;
    }
    let addresses = item["addresses"].as_array().into_iter().flatten().filter_map(Value::as_str);
    Some(json!({
        // Recurring meetings share their ID; the start makes each one unique.
        "key": format!("{}|{}", item["id"].as_str().unwrap_or(""), item["start_local"].as_str().unwrap_or("")),
        "start": item["start"],
        "end": item["end"],
        "subject": item["subject"],
        "location": item["location"],
        "domains": hw::mail_domains(addresses, ignore),
    }))
}

fn on_tick(now: i64) {
    let s = unsafe { &mut STATE };
    if s.read_at.is_some_and(|at| now - at < s.refresh_ms) {
        return;
    }
    let answer = hw::capability("outlook.calendar", &json!({ "days_back": s.days_back, "max_attendees": s.max_attendees }));
    let Some(answer) = answer.filter(|a| a["available"] == true) else {
        // Tried again next tick until Outlook is there.
        hw::status("degraded", "Outlook is not running (or runs as administrator); appointments are read when it is open");
        return;
    };
    let items: Vec<Value> = answer["items"].as_array().into_iter().flatten().filter_map(|i| convert(i, s.skip_free, &s.ignore)).collect();
    hw::status("running", &format!("Read {} appointments", items.len()));
    hw::emit(&json!({ "method": "calendar", "params": { "items": items } }));
    s.read_at = Some(now);
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for Outlook");
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

    fn item(extra: Value) -> Value {
        let mut base = json!({
            "id": "040000008200E0", "start": 1000, "start_local": "2026-11-03T09:00:00", "end": 2000,
            "subject": "Kick-off", "location": "Teams", "all_day": false, "busy_status": 2, "meeting_status": 1,
            "addresses": ["a@contoso.nl", "rik@hourwarden.com"],
        });
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        base
    }

    #[test]
    fn meetings_become_calendar_items() {
        let got = convert(&item(json!({})), true, &["hourwarden.com".into()]).unwrap();
        assert_eq!(got["key"], "040000008200E0|2026-11-03T09:00:00");
        assert_eq!(got["domains"], json!(["contoso.nl"]));
    }

    #[test]
    fn not_time_spent_is_left_out() {
        assert!(convert(&item(json!({ "all_day": true })), true, &[]).is_none());
        assert!(convert(&item(json!({ "busy_status": 0 })), true, &[]).is_none());
        assert!(convert(&item(json!({ "busy_status": 0 })), false, &[]).is_some());
        assert!(convert(&item(json!({ "meeting_status": 5 })), true, &[]).is_none());
    }
}

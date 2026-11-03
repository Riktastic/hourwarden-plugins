//! Outlook (classic): the subject and mail domains of the item in front, and
//! counts of mail sent and received per mail domain.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `outlook`
//! plugin. The app reads Outlook (capabilities `outlook.current_item` and
//! `outlook.mail_since`): only metadata, never a body. Which domains count
//! is decided here.

// Single-threaded wasm: the state lives in a static between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

struct State {
    poll_ms: i64,
    max_recipients: i64,
    count_mails: bool,
    count_every_ms: i64,
    count_days: i64,
    ignore: Vec<String>,
    now: i64,
    counted_at: Option<i64>,
    /// Per folder ("sent", "inbox"): the newest mail time counted (ms).
    counted_until: [Option<i64>; 2],
}

static mut STATE: State = State {
    poll_ms: 3000,
    max_recipients: 30,
    count_mails: true,
    count_every_ms: 5 * 60_000,
    count_days: 7,
    ignore: Vec::new(),
    now: 0,
    counted_at: None,
    counted_until: [None, None],
};

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.poll_ms = hw::poll_ms(settings, 3);
    s.max_recipients = hw::setting_i64(&settings["max_recipients"]).unwrap_or(30).clamp(1, 500);
    s.count_mails = settings["count_mails"].as_bool().unwrap_or(true);
    s.count_every_ms = hw::setting_i64(&settings["count_every_minutes"]).unwrap_or(5).clamp(1, 1440) * 60_000;
    s.count_days = hw::setting_i64(&settings["count_days"]).unwrap_or(7).clamp(1, 60);
    s.ignore = hw::string_list(&settings["ignore_domains"], &[]);
}

fn domains(addresses: &Value, ignore: &[String]) -> Vec<String> {
    hw::mail_domains(addresses.as_array().into_iter().flatten().filter_map(Value::as_str), ignore)
}

fn on_foreground(event: &Value) {
    if hw::app_name(event["exe"].as_str().unwrap_or("")) != "outlook" {
        return;
    }
    let s = unsafe { &STATE };
    let Some(answer) = hw::capability("outlook.current_item", &json!({ "max_recipients": s.max_recipients })) else {
        hw::status("degraded", "Outlook could not be read");
        return;
    };
    if answer["available"] != true {
        hw::status("degraded", "Outlook is running but cannot be reached; is one of them started as administrator?");
        return;
    }
    let item = &answer["item"];
    if item.is_object() {
        hw::emit(&json!({
            "method": "detail",
            "params": {
                "hwnd": event["hwnd"],
                "exe": "outlook.exe",
                "document": item["subject"],
                "mail_domains": domains(&item["addresses"], &s.ignore),
            }
        }));
    }
    hw::status("running", "Reading the selected Outlook item");
}

/// Reports every mail in Sent Items and the Inbox newer than the last check
/// (at the start: the last few days). The app counts each key once.
fn count_mails() {
    let s = unsafe { &mut STATE };
    let start = s.now - s.count_days * DAY_MS;
    let mut total = 0;
    for (i, (folder, metric)) in [("sent", "mail_sent"), ("inbox", "mail_received")].into_iter().enumerate() {
        let since = s.counted_until[i].map_or(start, |t| t.max(start));
        let arg = json!({ "folder": folder, "since": since, "max_recipients": s.max_recipients });
        let Some(answer) = hw::capability("outlook.mail_since", &arg) else { continue };
        if answer["available"] != true {
            return;
        }
        let mut newest = since;
        for mail in answer["items"].as_array().into_iter().flatten() {
            let at = mail["at"].as_i64().unwrap_or(0);
            newest = newest.max(at);
            let found = domains(&mail["addresses"], &s.ignore);
            if !found.is_empty() {
                let key = format!("{folder}:{}", mail["id"].as_str().unwrap_or(""));
                hw::emit(&json!({ "method": "metric", "params": { "metric": metric, "key": key, "at": at, "domains": found } }));
                total += 1;
            }
        }
        s.counted_until[i] = Some(newest);
    }
    if total > 0 {
        hw::log(&format!("Counted {total} new mails"));
    }
}

fn on_tick(now: i64) {
    let s = unsafe { &mut STATE };
    s.now = now;
    if s.count_mails && s.counted_at.is_none_or(|at| now - at >= s.count_every_ms) {
        s.counted_at = Some(now);
        count_mails();
    }
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for Outlook to be in front");
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();

fn on_event(event: Value) {
    match event["kind"].as_str() {
        Some("tick") => on_tick(event["now"].as_i64().unwrap_or(0)),
        Some("settings") => return configure(&event["settings"]),
        _ => {}
    }
    // The selected mail changes without the window title changing.
    if let Some(fg) = unsafe { POLL.due(&event, STATE.poll_ms) } {
        on_foreground(&fg);
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_domains_are_left_out() {
        let addresses = json!(["klant@contoso.nl", "rik@hourwarden.com", "/O=EXCHANGE/CN=X", "b@contoso.nl"]);
        assert_eq!(domains(&addresses, &["hourwarden.com".into()]), vec!["contoso.nl"]);
    }
}

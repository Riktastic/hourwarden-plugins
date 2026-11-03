//! Wi-Fi and VPN: the network the computer is connected to. Link a
//! customer's office Wi-Fi or VPN to that customer; it is used only when
//! nothing else decides the customer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `networks`
//! plugin. The app lists the active connections (capability
//! `net.connections`); this plugin chooses which one to report.

// Single-threaded wasm: the state lives in statics between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

struct State {
    prefer_vpn: bool,
    ignore: Vec<String>,
    poll_ms: i64,
    /// When the network was last read and last reported (ms), and what.
    polled_at: Option<i64>,
    sent_at: i64,
    last: Option<String>,
}

static mut STATE: State = State { prefer_vpn: true, ignore: Vec::new(), poll_ms: 30_000, polled_at: None, sent_at: 0, last: None };

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.prefer_vpn = settings["prefer_vpn"].as_bool().unwrap_or(true);
    s.ignore = settings["ignore_networks"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    s.poll_ms = settings["poll_seconds"].as_i64().unwrap_or(30).clamp(5, 600) * 1000;
    // Read again on the next tick with the new settings.
    s.polled_at = None;
}

/// The connection to report from the app's list: a VPN first when wanted,
/// then Wi-Fi, then a wired network; ignored names never.
fn choose<'a>(connections: &'a [Value], prefer_vpn: bool, ignore: &[String]) -> Option<(&'a str, &'a str)> {
    let usable: Vec<(&str, &str)> = connections
        .iter()
        .filter_map(|c| Some((c["name"].as_str()?, c["kind"].as_str()?)))
        .filter(|(name, _)| !ignore.iter().any(|i| i == name))
        .collect();
    let order: &[&str] = if prefer_vpn { &["vpn", "wifi", "wired"] } else { &["wifi", "wired"] };
    order.iter().find_map(|kind| usable.iter().find(|(_, k)| k == kind).copied())
}

fn on_tick(now: i64) {
    let s = unsafe { &mut STATE };
    if s.polled_at.is_some_and(|at| now - at < s.poll_ms) {
        return;
    }
    s.polled_at = Some(now);
    let Some(result) = hw::capability("net.connections", &Value::Null) else {
        hw::status("degraded", "Could not read the network");
        return;
    };
    let connections = result["connections"].as_array().cloned().unwrap_or_default();
    match choose(&connections, s.prefer_vpn, &s.ignore) {
        Some((name, kind)) => {
            let changed = s.last.as_deref() != Some(name);
            // Repeated every minute so the app knows it is still current.
            if changed || now - s.sent_at >= 60_000 {
                hw::emit(&json!({ "method": "network", "params": { "name": name, "kind": kind } }));
                if changed {
                    hw::log(&format!("Connected to {name} ({kind})"));
                    hw::status("running", &format!("Connected to {name}"));
                }
                s.last = Some(name.to_string());
                s.sent_at = now;
            }
        }
        None => {
            if s.last.take().is_some() || s.sent_at == 0 {
                hw::status("running", "Not connected");
            }
        }
    }
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Watching the network");
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

    fn conns() -> Vec<Value> {
        vec![
            json!({ "name": "Wired 1", "kind": "wired" }),
            json!({ "name": "Contoso-Guest", "kind": "wifi" }),
            json!({ "name": "Own VPN", "kind": "vpn" }),
        ]
    }

    #[test]
    fn vpn_first_when_preferred() {
        assert_eq!(choose(&conns(), true, &[]), Some(("Own VPN", "vpn")));
        assert_eq!(choose(&conns(), false, &[]), Some(("Contoso-Guest", "wifi")));
    }

    #[test]
    fn ignored_names_are_skipped() {
        assert_eq!(choose(&conns(), true, &["Own VPN".into()]), Some(("Contoso-Guest", "wifi")));
        assert_eq!(choose(&conns(), true, &["Own VPN".into(), "Contoso-Guest".into()]), Some(("Wired 1", "wired")));
        assert_eq!(choose(&[], true, &[]), None);
    }
}

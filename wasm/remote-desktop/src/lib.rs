//! Remote Desktop Manager and Remote Desktop Connection: the entry, folder
//! and server of the session in front.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `remote-desktop` plugin. In Remote Desktop Manager the selected tab's name
//! is the entry, and may carry its folder ("Contoso\Production\APP01"); the
//! server is the one the program holds a Remote Desktop connection to
//! (capabilities `ui.selected_tab` and `rdp.server`). Remote Desktop
//! Connection (mstsc) names the server first in its title.
//!
//! Unlike the PowerShell version it does not read entries through the
//! Devolutions PowerShell module, which itself needs PowerShell; folders come
//! from the tab names instead.

// Single-threaded wasm: the poll state lives in statics between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const RDM: &[&str] = &["remotedesktopmanager", "remotedesktopmanager64", "remotedesktopmanagerfree"];
/// Tabs of Remote Desktop Manager itself, not sessions.
const IGNORED_TABS: &[&str] = &["dashboard", "home", "start", "overview", "welcome", "navigation"];

/// Splits "Contoso\Production\APP01" into the folder and the entry name.
fn split_entry(entry: &str) -> (Option<&str>, &str) {
    match entry.rsplit_once('\\') {
        Some((folder, name)) => (Some(folder), name),
        None => (None, entry),
    }
}

/// The server in a Remote Desktop Connection title:
/// "app01.contoso.local - Remote Desktop Connection".
fn mstsc_server(title: &str) -> Option<&str> {
    let server = title.split(" - ").next()?.trim();
    let is_client = server.eq_ignore_ascii_case("Remote Desktop Connection") || server.eq_ignore_ascii_case("Verbinding met extern bureaublad");
    (!server.is_empty() && !is_client).then_some(server)
}

fn rdm_detail(event: &Value, exe: &str) -> Option<Value> {
    let tab = hw::capability("ui.selected_tab", &json!({ "hwnd": event["hwnd"] }))?;
    let entry = tab["name"].as_str()?.trim();
    if entry.is_empty() || IGNORED_TABS.contains(&entry.to_lowercase().as_str()) {
        return None;
    }
    let (folder, name) = split_entry(entry);
    let server = event["pid"]
        .as_u64()
        .filter(|p| *p > 0)
        .and_then(|pid| hw::capability("rdp.server", &json!({ "pid": pid })))
        .and_then(|s| s["server"].as_str().map(str::to_string));
    Some(json!({ "hwnd": event["hwnd"], "exe": exe, "document": name, "remote_folder": folder, "remote_server": server }))
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 3) };
    hw::status("running", "Waiting for a remote desktop window");
}

fn on_event(event: Value) {
    if event["kind"] == "settings" {
        unsafe { POLL_MS = hw::poll_ms(&event["settings"], 3) };
    } else if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        on_foreground(&fg);
    }
}

fn on_foreground(event: &Value) {
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let app = hw::app_name(&exe);
    let detail = if RDM.contains(&app.as_str()) {
        let d = rdm_detail(event, &exe);
        hw::status("running", "Reading Remote Desktop Manager tabs");
        d
    } else if app == "mstsc" {
        mstsc_server(event["title"].as_str().unwrap_or(""))
            .map(|server| json!({ "hwnd": event["hwnd"], "exe": exe, "document": server, "remote_server": server }))
    } else {
        None
    };
    if let Some(detail) = detail {
        hw::emit(&json!({ "method": "detail", "params": detail }));
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_and_titles() {
        assert_eq!(split_entry(r"Contoso\Production\APP01"), (Some(r"Contoso\Production"), "APP01"));
        assert_eq!(split_entry("APP01"), (None, "APP01"));
        assert_eq!(mstsc_server("app01.contoso.local - Remote Desktop Connection"), Some("app01.contoso.local"));
        assert_eq!(mstsc_server("Remote Desktop Connection"), None);
    }
}

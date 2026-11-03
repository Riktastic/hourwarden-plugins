//! Remote workplaces: the desktop or app you work in through Citrix Workspace,
//! Azure Virtual Desktop and Windows 365 (Windows App / Remote Desktop client)
//! and VMware Horizon. The workplace name usually carries the customer's name
//! and is matched against customer names and aliases. It reads only the title
//! of the window in front; nothing leaves the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `remote-workspaces` plugin.

// Single-threaded wasm: the poll state lives in statics between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

fn app_name(exe: &str) -> &str {
    exe.strip_suffix(".exe").unwrap_or(exe)
}

/// Title tails to drop for each client program (longest first so, e.g., the
/// VMware " Client" tail is removed before the shorter one).
fn suffixes(app: &str) -> Option<&'static [&'static str]> {
    Some(match app {
        "cdviewer" => &[" - Desktop Viewer"],
        "wfica32" | "wfica" => &[" (Remote)", " (Extern)", " - Citrix Workspace"],
        "msrdc" => &[" - Remote Desktop", " - Extern bureaublad"],
        "windows365" => &[" - Windows 365"],
        "windowsapp" => &[" - Windows App"],
        "vmware-view" | "vmware-remotemks" => &[" - VMware Horizon Client", " - VMware Horizon"],
        _ => return None,
    })
}

/// A client's own windows (start screen or settings), not a workplace.
fn is_client_window(name: &str) -> bool {
    matches!(
        name,
        "Citrix Workspace"
            | "Windows App"
            | "Windows 365"
            | "Remote Desktop"
            | "VMware Horizon"
            | "VMware Horizon Client"
            | "Microsoft Remote Desktop"
    )
}

/// The workplace name from a client's window title, or `None` for the client's
/// own windows.
fn workspace_name(exe: &str, title: &str) -> Option<String> {
    let sufs = suffixes(app_name(exe))?;
    if title.is_empty() {
        return None;
    }
    let mut name = title;
    for s in sufs {
        if let Some(stripped) = name.strip_suffix(s) {
            name = stripped;
            break;
        }
    }
    let name = name.trim();
    (!name.is_empty() && !is_client_window(name)).then(|| name.to_string())
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 2000;

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 2) };
    hw::status("running", "Waiting for a remote workplace to be in front");
}

fn on_event(event: Value) {
    if event["kind"] == "settings" {
        unsafe { POLL_MS = hw::poll_ms(&event["settings"], 2) };
    } else if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        on_foreground(&fg);
    }
}

fn on_foreground(event: &Value) {
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let title = event["title"].as_str().unwrap_or("");
    if let Some(name) = workspace_name(&exe, title) {
        hw::emit(&json!({
            "method": "detail",
            "params": { "hwnd": event["hwnd"], "exe": exe, "document": name, "remote_folder": name }
        }));
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citrix_and_rdp() {
        assert_eq!(workspace_name("cdviewer.exe", "CONTOSO-RDS - Desktop Viewer"), Some("CONTOSO-RDS".into()));
        assert_eq!(workspace_name("msrdc.exe", "Contoso Cloud PC - Remote Desktop"), Some("Contoso Cloud PC".into()));
        assert_eq!(workspace_name("wfica32.exe", "Contoso Apps (Remote)"), Some("Contoso Apps".into()));
    }

    #[test]
    fn horizon_client_tail_first() {
        assert_eq!(workspace_name("vmware-view.exe", "Fabrikam Desk - VMware Horizon Client"), Some("Fabrikam Desk".into()));
    }

    #[test]
    fn client_own_windows_ignored() {
        assert_eq!(workspace_name("windowsapp.exe", "Windows App"), None);
        assert_eq!(workspace_name("explorer.exe", "Something"), None);
    }
}

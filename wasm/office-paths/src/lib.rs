//! Office and Explorer: the full path of the document open in Word, Excel and
//! PowerPoint, and of the folder open in File Explorer, so customer folders
//! are recognised even when the title only shows the file name. Files in
//! OneDrive or SharePoint give their web address.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `office-paths` plugin. The app reads the path (capability
//! `office.document`); never the document's content.

// Single-threaded wasm: the poll state lives in statics between calls.
#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const APPS: &[&str] = &["winword", "excel", "powerpnt", "explorer"];

/// What to report: the path, or only the name while a document has not been
/// saved yet ("Document1").
fn detail(event: &Value, exe: &str, path: &str) -> Value {
    let key = if path.contains('\\') || path.contains("://") { "path" } else { "document" };
    json!({ "hwnd": event["hwnd"], "exe": exe, key: path })
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 3) };
    hw::status("running", "Waiting for Office or Explorer to be in front");
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
    if !APPS.contains(&hw::app_name(&exe).as_str()) {
        return;
    }
    let answer = hw::capability("office.document", &json!({ "exe": exe, "hwnd": event["hwnd"] }));
    if let Some(path) = answer.as_ref().and_then(|a| a["path"].as_str()).filter(|p| !p.is_empty()) {
        hw::emit(&json!({ "method": "detail", "params": detail(event, &exe, path) }));
        hw::status("running", "Reading document paths");
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_documents_give_a_path_new_ones_a_name() {
        let ev = json!({ "hwnd": 7 });
        assert_eq!(detail(&ev, "winword.exe", r"D:\Klanten\Contoso\Offerte.docx")["path"], r"D:\Klanten\Contoso\Offerte.docx");
        assert_eq!(detail(&ev, "excel.exe", "https://contoso.sharepoint.com/sites/x/Book.xlsx")["path"], "https://contoso.sharepoint.com/sites/x/Book.xlsx");
        assert_eq!(detail(&ev, "winword.exe", "Document1")["document"], "Document1");
    }
}

//! Admin tools: the server, database or domain you manage, from the title of
//! SQL Server Management Studio, WinSCP, FileZilla, Hyper-V and VMware consoles
//! and Windows management consoles (Active Directory, DNS). It reads only the
//! title of the window in front; nothing leaves the computer.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `admin-tools`
//! plugin.

use hourwarden_plugin as hw;
use hw::{json, Value};

fn app_name(exe: &str) -> &str {
    exe.strip_suffix(".exe").unwrap_or(exe)
}

/// Matches `[A-Za-z0-9][A-Za-z0-9-]*(\.[A-Za-z0-9-]+)*`.
fn is_hostname(s: &str) -> bool {
    if s.is_empty() || !s.as_bytes()[0].is_ascii_alphanumeric() {
        return false;
    }
    s.split('.').all(|label| !label.is_empty() && label.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'))
}

/// SSMS: "Query.sql - SQL01\PROD.Sales (CONTOSO\rik (53)) - ... Studio".
/// Splits a `server.database` target at the last dot.
fn ssms(title: &str) -> Option<Value> {
    let mut from = 0;
    while let Some(rel) = title[from..].find(" - ") {
        let pos = from + rel + 3;
        let rest = &title[pos..];
        let tend = rest.find([' ', '(', ')']).unwrap_or(rest.len());
        let target = &rest[..tend];
        if !target.is_empty() && rest[tend..].starts_with(" (") && rest[tend..].contains(')') {
            return Some(match target.rfind('.') {
                Some(i) if i > 0 => json!({ "remote_server": target[..i].to_lowercase(), "document": target[i + 1..] }),
                _ => json!({ "remote_server": target.to_lowercase() }),
            });
        }
        from = pos;
    }
    None
}

/// WinSCP: "/var/www - deploy@web01.contoso.nl - WinSCP".
fn winscp(title: &str) -> Option<Value> {
    let s = title.strip_suffix(" - WinSCP")?;
    let at = s.rfind('@')?;
    let user_start = s[..at].rfind(" - ").map_or(0, |i| i + 3);
    let user = &s[user_start..at];
    let host = &s[at + 1..];
    (!user.is_empty() && !user.contains(' ') && is_hostname(host)).then(|| json!({ "remote_server": host.to_lowercase() }))
}

/// FileZilla: "Contoso - sftp://deploy@ftp.contoso.nl - FileZilla".
fn filezilla(title: &str) -> Option<Value> {
    for scheme in ["sftp://", "ftps://", "sftps://", "ftp://"] {
        if let Some(i) = title.find(scheme) {
            let rest = &title[i + scheme.len()..];
            let rest = match rest.find('@') {
                Some(k) if !rest[..k].contains(['/', ' ']) => &rest[k + 1..],
                _ => rest,
            };
            let hend = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-')).unwrap_or(rest.len());
            let host = &rest[..hend];
            if is_hostname(host) {
                return Some(json!({ "remote_server": host.to_lowercase() }));
            }
        }
    }
    None
}

/// Hyper-V: "APP01 on HV-HOST01 - Virtual Machine Connection".
fn vmconnect(title: &str) -> Option<Value> {
    let on = title.find(" on ")?;
    let vm = title[..on].trim();
    let rest = &title[on + 4..];
    let host = &rest[..rest.find(" - ")?];
    (!vm.is_empty() && is_hostname(host)).then(|| json!({ "remote_server": vm, "remote_folder": host }))
}

/// VMware Remote Console: "APP01 - VMware Remote Console".
fn vmrc(title: &str) -> Option<Value> {
    let vm = title[..title.find(" - VMware Remote Console")?].trim();
    (!vm.is_empty()).then(|| json!({ "remote_server": vm }))
}

/// MMC consoles: the first dotted machine name in the title, skipping bare
/// IPv4-looking numbers unless they have four parts.
fn mmc(title: &str) -> Option<Value> {
    let b = title.as_bytes();
    let is_name = |c: u8| c.is_ascii_alphanumeric() || c == b'-';
    let mut i = 0;
    while i < b.len() {
        let delimited = i > 0 && matches!(b[i - 1], b'[' | b'\\' | b' ' | b'\t' | b':');
        if delimited && is_name(b[i]) {
            let start = i;
            while i < b.len() && (is_name(b[i]) || b[i] == b'.') {
                i += 1;
            }
            let name = &title[start..i];
            let next_ok = i == b.len() || matches!(b[i], b']' | b'\\' | b' ' | b'\t');
            if next_ok && valid_dotted(name) && !is_ipish_not_four(name) {
                return Some(json!({ "remote_server": name.to_lowercase() }));
            }
            continue;
        }
        i += 1;
    }
    None
}

/// A `label(.label)+` name with no empty labels.
fn valid_dotted(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() >= 2 && parts.iter().all(|p| !p.is_empty())
}

/// All-numeric dotted name that is not a 4-part IPv4 address.
fn is_ipish_not_four(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    let all_num = parts.iter().all(|p| p.bytes().all(|c| c.is_ascii_digit()));
    all_num && parts.len() != 4
}

fn get_admin_target(exe: &str, title: &str) -> Option<Value> {
    if title.is_empty() {
        return None;
    }
    match app_name(exe) {
        "ssms" => ssms(title),
        "winscp" => winscp(title),
        "filezilla" => filezilla(title),
        "vmconnect" => vmconnect(title),
        "vmrc" => vmrc(title),
        "mmc" => mmc(title),
        _ => None,
    }
}

fn on_init(_data: Value) {
    hw::status("running", "Waiting for an admin tool to be in front");
}

fn on_event(event: Value) {
    if event["kind"] != "foreground" {
        return;
    }
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let title = event["title"].as_str().unwrap_or("");
    if let Some(mut target) = get_admin_target(&exe, title) {
        if let Some(obj) = target.as_object_mut() {
            obj.insert("hwnd".into(), event["hwnd"].clone());
            obj.insert("exe".into(), Value::String(exe));
        }
        hw::emit(&json!({ "method": "detail", "params": target }));
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssms_server_and_database() {
        let t = "Query.sql - SQL01\\PROD.Sales (CONTOSO\\rik (53)) - Microsoft SQL Server Management Studio";
        assert_eq!(get_admin_target("ssms.exe", t), Some(json!({ "remote_server": "sql01\\prod", "document": "Sales" })));
    }

    #[test]
    fn winscp_and_filezilla() {
        assert_eq!(get_admin_target("winscp.exe", "/var/www - deploy@web01.contoso.nl - WinSCP"), Some(json!({ "remote_server": "web01.contoso.nl" })));
        assert_eq!(get_admin_target("filezilla.exe", "Contoso - sftp://deploy@ftp.contoso.nl - FileZilla"), Some(json!({ "remote_server": "ftp.contoso.nl" })));
    }

    #[test]
    fn hyperv_and_vmrc() {
        assert_eq!(get_admin_target("vmconnect.exe", "APP01 on HV-HOST01 - Virtual Machine Connection"), Some(json!({ "remote_server": "APP01", "remote_folder": "HV-HOST01" })));
        assert_eq!(get_admin_target("vmrc.exe", "APP01 - VMware Remote Console"), Some(json!({ "remote_server": "APP01" })));
    }

    #[test]
    fn mmc_machine_name() {
        assert_eq!(get_admin_target("mmc.exe", "DNS Manager - [DNS\\dc01.contoso.local]"), Some(json!({ "remote_server": "dc01.contoso.local" })));
    }
}

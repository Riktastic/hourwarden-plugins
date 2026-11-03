//! Web addresses: the domain of the page in front in Edge, Chrome, Firefox,
//! Brave and other browsers. Only the domain (portal.contoso.com) and
//! Microsoft tenant, subscription and workspace IDs are reported, never the
//! full address; they are matched like a mail domain.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `web-addresses` plugin. The app reads the address bar (capability
//! `browser.address`); what it says is worked out here.

// Single-threaded wasm: the state lives in a static between calls.
#![allow(static_mut_refs)]

use std::collections::HashSet;

use hourwarden_plugin as hw;
use hw::{json, Value};
use regex_lite::Regex;

const DEFAULT_BROWSERS: &[&str] = &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "vivaldi.exe", "opera.exe", "arc.exe"];
const GUID: &str = "[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}";
/// IDs logged once each; the list is cleared when it grows past this.
const MAX_SEEN: usize = 500;

struct Patterns {
    internal: Regex,
    scheme: Regex,
    host: Regex,
    sharepoint: Regex,
    power_bi: Regex,
    ids: Vec<(&'static str, Regex)>,
    workspace: Regex,
}

impl Patterns {
    fn new() -> Patterns {
        Patterns {
            internal: Regex::new(r"(?i)^(edge|chrome|brave|vivaldi|opera|about|file|data|javascript|view-source|devtools|chrome-extension|extension):").unwrap(),
            scheme: Regex::new(r"(?i)^[a-z][a-z0-9+.-]*://").unwrap(),
            host: Regex::new(r"^[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+$").unwrap(),
            sharepoint: Regex::new(r"^([a-z0-9]+)-(admin|my)\.sharepoint\.com$").unwrap(),
            power_bi: Regex::new(r"(^|\.)(powerbi\.com|fabric\.microsoft\.com)$").unwrap(),
            ids: vec![
                ("tenant", Regex::new(&format!(r"#@(?P<id>{GUID}|[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+)")).unwrap()),
                ("tenant", Regex::new(&format!(r"(?i)[?&#](ctid|tid|tenantId|tenant)=(?P<id>{GUID})")).unwrap()),
                ("subscription", Regex::new(&format!(r"(?i)/subscriptions/(?P<id>{GUID})")).unwrap()),
            ],
            workspace: Regex::new(&format!(r"(?i)/groups/(?P<id>{GUID})")).unwrap(),
        }
    }

    /// The domain in an address-bar text, or `None` for browser pages, local
    /// files and typed search words.
    fn web_host(&self, address: &str) -> Option<String> {
        let a = address.trim();
        if a.is_empty() || self.internal.is_match(a) {
            return None;
        }
        let a = self.scheme.replace(a, "");
        let host = a.split(['/', '?', '#']).next().unwrap_or("");
        let host = host.split_once('@').map_or(host, |(_, h)| h);
        let host = match host.rsplit_once(':') {
            Some((h, port)) if !port.is_empty() && port.bytes().all(|c| c.is_ascii_digit()) => h,
            _ => host,
        };
        if !self.host.is_match(host) {
            return None;
        }
        let host = host.to_lowercase();
        Some(host.strip_prefix("www.").map(str::to_string).unwrap_or(host))
    }

    /// Microsoft IDs in an address as (kind, id): tenants in Azure, Entra,
    /// Power BI and Fabric addresses, Azure subscriptions, and Power BI and
    /// Fabric workspaces.
    fn microsoft_ids(&self, address: &str) -> Vec<(&'static str, String)> {
        let mut patterns: Vec<(&str, &Regex)> = self.ids.iter().map(|(k, r)| (*k, r)).collect();
        if self.web_host(address).is_some_and(|h| self.power_bi.is_match(&h)) {
            patterns.push(("workspace", &self.workspace));
        }
        let mut out: Vec<(&str, String)> = Vec::new();
        for (kind, pattern) in patterns {
            for caps in pattern.captures_iter(address) {
                let id = caps["id"].to_lowercase();
                if !out.iter().any(|(_, seen)| *seen == id) {
                    out.push((kind, id));
                }
            }
        }
        out
    }

    /// What an address says: its domain (SharePoint admin and OneDrive hosts
    /// as the customer's SharePoint host) plus the Microsoft IDs in it.
    fn domains(&self, address: &str) -> Vec<String> {
        let Some(host) = self.web_host(address) else { return Vec::new() };
        let mut out = vec![self.sharepoint.replace(&host, "$1.sharepoint.com").into_owned()];
        for (_, id) in self.microsoft_ids(address) {
            if !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }
}

struct State {
    browsers: Vec<String>,
    ignore: Vec<String>,
    patterns: Option<Patterns>,
    seen: HashSet<String>,
}

static mut STATE: Option<State> = None;

fn configure(settings: &Value) {
    let state = unsafe { STATE.get_or_insert_with(|| State { browsers: Vec::new(), ignore: Vec::new(), patterns: None, seen: HashSet::new() }) };
    state.browsers = hw::string_list(&settings["browsers"], DEFAULT_BROWSERS).iter().map(|b| hw::app_name(b)).collect();
    state.ignore = hw::string_list(&settings["ignore_domains"], &[]);
    unsafe { POLL_MS = hw::poll_ms(settings, 3) };
}

fn on_foreground(event: &Value) {
    let Some(s) = (unsafe { STATE.as_mut() }) else { return };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    if !s.browsers.contains(&hw::app_name(&exe)) {
        return;
    }
    let Some(address) = hw::capability("browser.address", &json!({ "hwnd": event["hwnd"] })).and_then(|a| a["address"].as_str().map(str::to_string))
    else {
        hw::status("degraded", "The address bar of this browser could not be found");
        return;
    };
    let p = s.patterns.get_or_insert_with(Patterns::new);
    // Each Microsoft ID is logged once, so it can be copied to the right customer.
    let host = p.web_host(&address).unwrap_or_default();
    for (kind, id) in p.microsoft_ids(&address) {
        if s.seen.len() > MAX_SEEN {
            s.seen.clear();
        }
        if s.seen.insert(id.clone()) {
            hw::log(&format!("Seen {kind} {id} on {host}"));
        }
    }
    let domains: Vec<String> = p.domains(&address).into_iter().filter(|d| !hw::domain_ignored(d, &s.ignore)).collect();
    if !domains.is_empty() {
        hw::emit(&json!({ "method": "detail", "params": { "hwnd": event["hwnd"], "exe": exe, "mail_domains": domains } }));
    }
    hw::status("running", "Reading the address bar");
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for a browser to be in front");
}

fn on_event(event: Value) {
    if event["kind"] == "settings" {
        configure(&event["settings"]);
    } else if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        on_foreground(&fg);
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts() {
        let p = Patterns::new();
        assert_eq!(p.web_host("https://www.Contoso.com:8443/path?q=1"), Some("contoso.com".into()));
        assert_eq!(p.web_host("portal.contoso.com/login"), Some("portal.contoso.com".into()));
        assert_eq!(p.web_host("edge://settings"), None);
        assert_eq!(p.web_host("how to bake bread"), None);
    }

    #[test]
    fn microsoft_ids_and_sharepoint() {
        let p = Patterns::new();
        let tenant = "11111111-2222-3333-4444-555555555555";
        let sub = "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE";
        let azure = format!("https://portal.azure.com/#@contoso.onmicrosoft.com/resource/subscriptions/{sub}/overview");
        assert_eq!(p.domains(&azure), vec!["portal.azure.com".to_string(), "contoso.onmicrosoft.com".into(), sub.to_lowercase()]);
        let ws = "99999999-8888-7777-6666-555555555555";
        let pbi = format!("https://app.powerbi.com/groups/{ws}/reports?ctid={tenant}");
        assert_eq!(p.domains(&pbi), vec!["app.powerbi.com".to_string(), tenant.into(), ws.into()]);
        assert_eq!(p.domains("https://contoso-admin.sharepoint.com/_layouts"), vec!["contoso.sharepoint.com"]);
        // Workspaces only count on Power BI and Fabric.
        assert_eq!(p.domains(&format!("https://example.com/groups/{ws}")), vec!["example.com"]);
    }
}

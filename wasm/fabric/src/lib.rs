//! Microsoft Fabric and Power BI: what you have open and what you do with it.
//!
//! In the browser, the address of a Fabric or Power BI page says the kind of
//! item (`/groups/<workspace>/reports/<id>`, `/datasets/<id>`,
//! `/synapsenotebooks/<id>`…) and sometimes the view (`/modeling/<id>/modelView`
//! is editing a semantic model). The tab title gives the item's name. Desktop
//! tools are recognised by program and title: Power BI Desktop, Tabular
//! Editor, DAX Studio, and SSMS connected to a workspace's XMLA endpoint
//! (`powerbi://api.powerbi.com/v1.0/myorg/<Workspace>`).
//!
//! The plugin reports it as the document of the window ("Sales · semantic
//! model · editing the model"), which the activity log and the entry show.
//! The address is read only on Power BI and Fabric pages and never sent
//! anywhere; tenant and workspace IDs are left to the web-addresses plugin.

#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const DEFAULT_BROWSERS: &[&str] = &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "vivaldi.exe", "opera.exe", "arc.exe"];

/// What is open: the item's kind, its name when known, and what you do.
#[derive(Debug, Clone, PartialEq)]
struct Work {
    kind: &'static str,
    name: Option<String>,
    doing: Option<&'static str>,
}

/// The kind of work each kind of item is, for the activity log's totals.
fn category(kind: &str) -> &'static str {
    match kind {
        "report" | "paginated report" | "dashboard" | "scorecard" | "app" | "Power BI Desktop file" => "Reporting",
        "semantic model" | "DAX queries" => "Data modelling",
        "notebook" | "lakehouse" | "Spark job definition" | "environment" | "GraphQL API" => "Data engineering",
        "data pipeline" | "dataflow" | "copy job" | "mirrored database" => "Data integration",
        "warehouse" | "SQL analytics endpoint" | "SQL database" => "Data warehousing",
        "eventhouse" | "eventstream" | "KQL database" | "KQL queryset" | "real-time dashboard" | "Activator" => "Real-time analytics",
        "ML model" | "ML experiment" => "Data science",
        "workspace" | "workspace item" | "admin portal" | "capacity settings" | "monitoring hub" | "deployment pipeline" | "OneLake catalog" | "variable library" => {
            "Administration"
        }
        _ => "Fabric",
    }
}

impl Work {
    fn category(&self) -> &'static str {
        category(self.kind)
    }

    fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(n) = self.name.as_deref().filter(|n| !n.is_empty()) {
            parts.push(n.to_string());
        }
        parts.push(self.kind.to_string());
        if let Some(d) = self.doing {
            parts.push(d.to_string());
        }
        parts.join(" · ")
    }
}

/// Fabric and Power BI item kinds by the path segment after the workspace.
const ITEMS: &[(&str, &str)] = &[
    ("reports", "report"),
    ("rdlreports", "paginated report"),
    ("datasets", "semantic model"),
    ("semanticmodels", "semantic model"),
    ("modeling", "semantic model"),
    ("dashboards", "dashboard"),
    ("dataflows", "dataflow"),
    ("dataflows-gen2", "dataflow"),
    ("datapipelines", "data pipeline"),
    ("pipelines", "data pipeline"),
    ("synapsenotebooks", "notebook"),
    ("notebooks", "notebook"),
    ("lakehouses", "lakehouse"),
    ("datawarehouses", "warehouse"),
    ("warehouses", "warehouse"),
    ("sqlanalyticsendpoints", "SQL analytics endpoint"),
    ("lakewarehouses", "SQL analytics endpoint"),
    ("sqldatabases", "SQL database"),
    ("mirroreddatabases", "mirrored database"),
    ("kqldatabases", "KQL database"),
    ("kqlquerysets", "KQL queryset"),
    ("kqldashboards", "real-time dashboard"),
    ("eventhouses", "eventhouse"),
    ("eventstreams", "eventstream"),
    ("reflexes", "Activator"),
    ("environments", "environment"),
    ("sparkjobdefinitions", "Spark job definition"),
    ("mlmodels", "ML model"),
    ("mlexperiments", "ML experiment"),
    ("copyjobs", "copy job"),
    ("graphqlapis", "GraphQL API"),
    ("variablelibraries", "variable library"),
    ("scorecards", "scorecard"),
    ("metrics", "scorecard"),
    ("apps", "app"),
];

/// Pages outside a workspace item, by the first path segment.
const PAGES: &[(&str, &str)] = &[
    ("home", "home"),
    ("admin-portal", "admin portal"),
    ("monitoringhub", "monitoring hub"),
    ("datahub", "OneLake catalog"),
    ("onelake", "OneLake catalog"),
    ("deploymentpipelines", "deployment pipeline"),
    ("browse", "browse"),
    ("capacities", "capacity settings"),
];

fn is_fabric_host(host: &str) -> bool {
    let h = host.to_lowercase();
    ["app.powerbi.com", "app.fabric.microsoft.com", "msit.powerbi.com", "app.powerbigov.us", "app.powerbi.cn"].contains(&h.as_str())
        || h.ends_with(".powerbi.com")
        || h.ends_with(".fabric.microsoft.com")
}

/// Host and path (lower-cased segments) of an address, without query.
fn host_and_path(address: &str) -> Option<(String, Vec<String>, String)> {
    let a = address.trim();
    let a = a.split_once("://").map_or(a, |(_, rest)| rest);
    let (host, rest) = a.split_once('/').unwrap_or((a, ""));
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let path = path.split('#').next().unwrap_or("");
    let segments = path.split('/').filter(|s| !s.is_empty()).map(|s| s.to_lowercase()).collect();
    Some((host.to_lowercase(), segments, query.to_lowercase()))
}

/// The item's name from a browser tab title: "Sales - Power BI - Personal -
/// Microsoft Edge" -> "Sales".
fn name_from_title(title: &str) -> Option<String> {
    let t = title.trim();
    for marker in [" - Power BI", " - Microsoft Fabric", " - Fabric", " | Power BI", " | Microsoft Fabric"] {
        if let Some(i) = t.find(marker) {
            let name = t[..i].trim();
            return (!name.is_empty() && !name.eq_ignore_ascii_case("home")).then(|| name.to_string());
        }
    }
    None
}

/// What a Fabric or Power BI address (and its tab title) says you work on.
fn from_address(address: &str, title: &str) -> Option<Work> {
    let (host, seg, query) = host_and_path(address)?;
    if !is_fabric_host(&host) {
        return None;
    }
    let name = name_from_title(title);
    // /groups/<workspace>/<kind>/<id>/<view…>
    if seg.first().map(String::as_str) == Some("groups") && seg.len() >= 2 {
        let rest = &seg[2..];
        let Some(kind_seg) = rest.first() else {
            return Some(Work { kind: "workspace", name, doing: None });
        };
        if kind_seg == "list" || kind_seg == "lineage" {
            let doing = (kind_seg == "lineage").then_some("lineage view");
            return Some(Work { kind: "workspace", name, doing });
        }
        let Some((_, kind)) = ITEMS.iter().find(|(s, _)| s == kind_seg) else {
            return Some(Work { kind: "workspace item", name, doing: None });
        };
        let views: Vec<&str> = rest.iter().skip(2).map(String::as_str).collect();
        let doing = if kind_seg == "modeling" || views.iter().any(|v| *v == "modelview" || *v == "model") {
            Some("editing the model")
        } else if *kind == "semantic model" && views.iter().any(|v| *v == "details") {
            Some("details")
        } else if *kind == "semantic model" && views.iter().any(|v| *v == "settings") {
            Some("settings")
        } else if *kind == "report" && (query.contains("action=edit") || query.contains("edit=true") || views.iter().any(|v| *v == "edit")) {
            Some("editing")
        } else if *kind == "report" && views.iter().any(|v| *v == "create") {
            Some("creating")
        } else {
            None
        };
        return Some(Work { kind, name, doing });
    }
    let first = seg.first().map(String::as_str).unwrap_or("home");
    let kind = PAGES.iter().find(|(s, _)| *s == first).map(|(_, k)| *k).unwrap_or("Power BI");
    Some(Work { kind, name: if kind == "home" { None } else { name }, doing: None })
}

/// Desktop tools for Power BI and Fabric, by program and window title.
fn from_desktop(exe: &str, title: &str) -> Option<Work> {
    let app = hw::app_name(exe);
    let title = title.trim();
    let before = |marker: &str| title.find(marker).map(|i| title[..i].trim().to_string()).filter(|s| !s.is_empty());
    match app.as_str() {
        "pbidesktop" => Some(Work { kind: "Power BI Desktop file", name: before(" - Power BI Desktop").or_else(|| Some(title.to_string()).filter(|t| !t.is_empty())), doing: None }),
        "tabulareditor" | "tabulareditor3" => {
            let name = title.split_once(" - ").map(|(_, rest)| rest.trim().to_string()).filter(|s| !s.is_empty());
            Some(Work { kind: "semantic model", name: name.map(|n| xmla_workspace(&n).map(|w| format!("workspace {w}")).unwrap_or(n)), doing: Some("editing in Tabular Editor") })
        }
        "daxstudio" => {
            let name = xmla_workspace(title).map(|w| format!("workspace {w}"));
            Some(Work { kind: "DAX queries", name, doing: Some("in DAX Studio") })
        }
        "ssms" => xmla_workspace(title).map(|w| Work { kind: "semantic model", name: Some(format!("workspace {w}")), doing: Some("XMLA in SSMS") }),
        _ => None,
    }
}

/// The workspace in an XMLA endpoint: powerbi://api.powerbi.com/v1.0/myorg/Finance -> Finance.
fn xmla_workspace(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let i = lower.find("powerbi://")?;
    let rest = &text[i..];
    let after = rest.split_once("/myorg/").map(|(_, w)| w).or_else(|| rest.rsplit_once('/').map(|(_, w)| w))?;
    let end = after.find([')', ']', ' ', '"', ';']).unwrap_or(after.len());
    let ws = after[..end].trim().replace("%20", " ");
    (!ws.is_empty()).then_some(ws)
}

struct State {
    browsers: Vec<String>,
    last: Option<String>,
}

static mut STATE: State = State { browsers: Vec::new(), last: None };
static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.browsers = hw::string_list(&settings["browsers"], DEFAULT_BROWSERS).iter().map(|b| hw::app_name(b)).collect();
    unsafe { POLL_MS = hw::poll_ms(settings, 3) };
}

fn on_foreground(event: &Value) {
    let s = unsafe { &mut STATE };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let title = event["title"].as_str().unwrap_or("");
    let work = if s.browsers.contains(&hw::app_name(&exe)) {
        // Only Power BI and Fabric pages are worth reading the address for.
        let lower = title.to_lowercase();
        if !(lower.contains("power bi") || lower.contains("fabric")) {
            return;
        }
        let Some(address) = hw::capability("browser.address", &json!({ "hwnd": event["hwnd"] })).and_then(|a| a["address"].as_str().map(str::to_string)) else {
            hw::status("degraded", "The address bar of this browser could not be found");
            return;
        };
        from_address(&address, title)
    } else {
        from_desktop(&exe, title)
    };
    let Some(work) = work else { return };
    let document = work.describe();
    if s.last.as_deref() != Some(document.as_str()) {
        hw::log(&format!("Now: {document}"));
        s.last = Some(document.clone());
    }
    hw::emit(&json!({ "method": "detail", "params": { "hwnd": event["hwnd"], "exe": exe, "document": document, "category": work.category() } }));
    hw::status("running", "Following Fabric and Power BI");
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for Fabric or Power BI to be in front");
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

    const WS: &str = "1a2b3c4d-0000-1111-2222-333344445555";
    const ID: &str = "9f8e7d6c-aaaa-bbbb-cccc-ddddeeeeffff";

    fn at(path: &str, title: &str) -> String {
        from_address(&format!("https://app.powerbi.com/groups/{WS}/{path}?ctid=abc&experience=power-bi"), title).unwrap().describe()
    }

    #[test]
    fn items_and_what_you_do() {
        assert_eq!(at(&format!("reports/{ID}/ReportSection1"), "Sales overview - Power BI - Work - Microsoft Edge"), "Sales overview · report");
        assert_eq!(at(&format!("datasets/{ID}/details"), "Sales - Power BI"), "Sales · semantic model · details");
        assert_eq!(at(&format!("modeling/{ID}/modelView"), "Sales - Power BI"), "Sales · semantic model · editing the model");
        assert_eq!(at(&format!("synapsenotebooks/{ID}"), "Load customers - Microsoft Fabric"), "Load customers · notebook");
        assert_eq!(at(&format!("lakehouses/{ID}"), "Bronze - Fabric"), "Bronze · lakehouse");
        assert_eq!(at(&format!("datapipelines/{ID}"), "Nightly load - Microsoft Fabric"), "Nightly load · data pipeline");
        assert_eq!(at("list", "Finance - Power BI"), "Finance · workspace");
        assert_eq!(at(&format!("somethingnew/{ID}"), "X - Microsoft Fabric"), "X · workspace item");
    }

    #[test]
    fn every_kind_has_a_category() {
        for (_, kind) in ITEMS.iter().chain(PAGES.iter()) {
            let c = category(kind);
            assert!(c != "Fabric" || ["home", "browse"].contains(kind), "{kind} has no category");
        }
        for kind in ["Power BI Desktop file", "DAX queries"] {
            assert_ne!(category(kind), "Fabric", "{kind}");
        }
        let w = from_address(&format!("https://app.powerbi.com/groups/{WS}/modeling/{ID}/modelView"), "Sales - Power BI").unwrap();
        assert_eq!(w.category(), "Data modelling");
        assert_eq!(from_desktop("tabulareditor3.exe", "Tabular Editor 3 - Model.bim").unwrap().category(), "Data modelling");
    }

    #[test]
    fn pages_hosts_and_other_sites() {
        let fabric = |a: &str| from_address(a, "Monitor - Microsoft Fabric").map(|w| w.describe());
        assert_eq!(fabric("https://app.fabric.microsoft.com/monitoringhub?experience=fabric").as_deref(), Some("Monitor · monitoring hub"));
        assert_eq!(fabric("https://app.powerbi.com/home").as_deref(), Some("home"));
        assert_eq!(fabric("app.powerbi.com/admin-portal/tenantSettings").as_deref(), Some("Monitor · admin portal"));
        assert_eq!(fabric("https://portal.azure.com/#home"), None, "not Fabric");
        assert_eq!(fabric("https://example.com/groups/x/reports/y"), None);
    }

    #[test]
    fn desktop_tools() {
        let d = |exe: &str, title: &str| from_desktop(exe, title).map(|w| w.describe());
        assert_eq!(d("PBIDesktop.exe", "Sales - Power BI Desktop").as_deref(), Some("Sales · Power BI Desktop file"));
        assert_eq!(d("TabularEditor3.exe", "Tabular Editor 3 - Sales (powerbi://api.powerbi.com/v1.0/myorg/Finance)").as_deref(), Some("workspace Finance · semantic model · editing in Tabular Editor"));
        assert_eq!(d("TabularEditor.exe", "Tabular Editor - Model.bim").as_deref(), Some("Model.bim · semantic model · editing in Tabular Editor"));
        assert_eq!(d("DaxStudio.exe", "DAX Studio - Query1.dax - powerbi://api.powerbi.com/v1.0/myorg/Sales%20Team").as_deref(), Some("workspace Sales Team · DAX queries · in DAX Studio"));
        assert_eq!(d("Ssms.exe", "SQLQuery1.sql - powerbi://api.powerbi.com/v1.0/myorg/Finance.Sales (rik) - Microsoft SQL Server Management Studio").as_deref(), Some("workspace Finance.Sales · semantic model · XMLA in SSMS"));
        assert_eq!(d("ssms.exe", "SQLQuery1.sql - SQL01.Sales - Microsoft SQL Server Management Studio"), None, "a normal SQL Server");
        assert_eq!(d("winword.exe", "Report - Word"), None);
    }
}

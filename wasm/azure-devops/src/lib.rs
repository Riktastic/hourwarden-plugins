//! Azure DevOps: what you have open and what you do with it.
//!
//! The address says the area (`/_workitems/edit/1234`, `/_git/web/pullrequest/42`,
//! `/_build/results?buildId=…`, `/_wiki/…`) and the organisation and project
//! (`dev.azure.com/<org>/<project>/…`, or `<org>.visualstudio.com/<project>/…`).
//! The tab title gives the name ("Bug 1234: Login fails - Boards").
//!
//! The plugin reports it as the document of the window, with a kind of work
//! for the activity log's totals. A work item is also offered as the task.

#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const DEFAULT_BROWSERS: &[&str] = &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "vivaldi.exe", "opera.exe", "arc.exe"];

#[derive(Debug, Clone, PartialEq)]
struct Work {
    kind: &'static str,
    category: &'static str,
    name: Option<String>,
    /// "Contoso/Web": the organisation and project.
    place: Option<String>,
    /// A work item's number, offered as the task.
    work_item: Option<String>,
}

impl Work {
    fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(n) = self.name.as_deref().filter(|n| !n.is_empty()) {
            parts.push(n.to_string());
        }
        parts.push(self.kind.to_string());
        if let Some(p) = &self.place {
            parts.push(p.clone());
        }
        parts.join(" · ")
    }
}

const PLANNING: &str = "Planning";
const REVIEW: &str = "Code review";
const DEVELOPMENT: &str = "Development";
const CICD: &str = "CI/CD";
const TESTING: &str = "Testing";
const DOCS: &str = "Documentation";
const ADMIN: &str = "Administration";

/// The area by its `_segment`, with the kind and kind of work.
fn area(segment: &str, rest: &[String], query: &str) -> (&'static str, &'static str) {
    let has = |s: &str| rest.iter().any(|r| r == s);
    match segment {
        "_workitems" => ("work item", PLANNING),
        "_boards" if has("taskboard") => ("sprint board", PLANNING),
        "_boards" if has("backlog") => ("backlog", PLANNING),
        "_boards" => ("board", PLANNING),
        "_backlogs" => ("backlog", PLANNING),
        "_sprints" => ("sprint", PLANNING),
        "_queries" => ("query", PLANNING),
        "_dashboards" => ("dashboard", PLANNING),
        "_delivery" | "_deliveryplans" => ("delivery plan", PLANNING),
        "_git" if has("pullrequest") => ("pull request", REVIEW),
        "_git" if has("pullrequests") => ("pull requests", REVIEW),
        "_git" if has("pullrequestcreate") => ("new pull request", DEVELOPMENT),
        "_git" if has("commit") || has("commits") => ("commits", DEVELOPMENT),
        "_git" if has("branches") => ("branches", DEVELOPMENT),
        "_git" => ("repository", DEVELOPMENT),
        "_search" => ("code search", DEVELOPMENT),
        "_build" if query.contains("buildid=") || has("results") => ("pipeline run", CICD),
        "_build" => ("pipeline", CICD),
        "_release" | "_releaseprogress" | "_releasedefinition" => ("release", CICD),
        "_environments" => ("environment", CICD),
        "_library" => ("pipeline library", CICD),
        "_testplans" | "_testmanagement" | "_test" => ("test plan", TESTING),
        "_wiki" => ("wiki page", DOCS),
        "_artifacts" | "_packaging" => ("artifacts", CICD),
        "_settings" | "_admin" => ("settings", ADMIN),
        _ => ("project", PLANNING),
    }
}

fn split_address(address: &str) -> (String, Vec<String>, String) {
    let a = address.trim();
    let a = a.split_once("://").map_or(a, |(_, r)| r);
    let (host, rest) = a.split_once('/').unwrap_or((a, ""));
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let path = path.split('#').next().unwrap_or("");
    let seg = path.split('/').filter(|s| !s.is_empty()).map(|s| s.replace("%20", " ")).collect();
    (host.to_lowercase(), seg, query.to_lowercase())
}

/// The name from the tab title: "Bug 1234: Login fails - Boards - Personal -
/// Microsoft Edge" -> "Bug 1234: Login fails".
fn name_from_title(title: &str) -> Option<String> {
    let t = title.trim();
    for marker in [" - Boards", " - Repos", " - Pipelines", " - Test Plans", " - Artifacts", " - Wiki", " - Overview", " - Azure DevOps", " - Settings"] {
        if let Some(i) = t.find(marker) {
            let name = t[..i].trim();
            return (!name.is_empty()).then(|| name.to_string());
        }
    }
    None
}

/// What an Azure DevOps address (and its tab title) says you work on.
fn from_address(address: &str, title: &str) -> Option<Work> {
    let (host, seg, query) = split_address(address);
    // dev.azure.com/<org>/<project>/… or <org>.visualstudio.com/<project>/…
    let (org, rest): (String, &[String]) = if host == "dev.azure.com" {
        (seg.first()?.clone(), &seg[1..])
    } else if let Some(org) = host.strip_suffix(".visualstudio.com") {
        (org.to_string(), &seg[..])
    } else {
        return None;
    };
    let name = name_from_title(title);
    // The project is the first segment that is not an area (an area starts with _).
    let project = rest.first().filter(|s| !s.starts_with('_')).cloned();
    let after = if project.is_some() { &rest[1..] } else { rest };
    let place = Some(match &project {
        Some(p) => format!("{org}/{p}"),
        None => org.clone(),
    });
    let Some(area_seg) = after.iter().find(|s| s.starts_with('_')) else {
        let kind = if project.is_some() { "project" } else { "organisation" };
        return Some(Work { kind, category: if project.is_some() { PLANNING } else { ADMIN }, name, place, work_item: None });
    };
    let lower: Vec<String> = after.iter().map(|s| s.to_lowercase()).collect();
    let (kind, category) = area(&area_seg.to_lowercase(), &lower, &query);
    // _workitems/edit/1234, or ?workitem=1234 on a board.
    let work_item = if kind == "work item" {
        lower.iter().skip_while(|s| *s != "edit").nth(1).filter(|n| n.bytes().all(|b| b.is_ascii_digit())).cloned()
    } else {
        query.split('&').find_map(|kv| kv.strip_prefix("workitem=")).filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())).map(str::to_string)
    };
    Some(Work { kind, category, name, place, work_item })
}

/// The title of a work item for the task: "Bug 1234: Login fails" -> "Login fails".
fn work_item_title(name: &str) -> String {
    name.split_once(": ").map(|(_, t)| t.trim().to_string()).unwrap_or_default()
}

struct State {
    browsers: Vec<String>,
    offer: bool,
    last: Option<String>,
}

static mut STATE: State = State { browsers: Vec::new(), offer: true, last: None };
static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn configure(settings: &Value) {
    let s = unsafe { &mut STATE };
    s.browsers = hw::string_list(&settings["browsers"], DEFAULT_BROWSERS).iter().map(|b| hw::app_name(b)).collect();
    s.offer = settings["offer_work_items"].as_bool().unwrap_or(true);
    unsafe { POLL_MS = hw::poll_ms(settings, 3) };
}

fn on_foreground(event: &Value) {
    let s = unsafe { &mut STATE };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let title = event["title"].as_str().unwrap_or("");
    if !s.browsers.contains(&hw::app_name(&exe)) {
        return;
    }
    let lower = title.to_lowercase();
    let devops_title = ["boards", "repos", "pipelines", "azure devops", "test plans", "artifacts", "wiki", "overview"].iter().any(|w| lower.contains(w));
    if !devops_title {
        return;
    }
    let Some(address) = hw::capability("browser.address", &json!({ "hwnd": event["hwnd"] })).and_then(|a| a["address"].as_str().map(str::to_string)) else {
        hw::status("degraded", "The address bar of this browser could not be found");
        return;
    };
    let Some(work) = from_address(&address, title) else { return };
    let document = work.describe();
    if s.last.as_deref() != Some(document.as_str()) {
        hw::log(&format!("Now: {document}"));
        s.last = Some(document.clone());
    }
    let mut params = json!({ "hwnd": event["hwnd"], "exe": exe, "document": document, "category": work.category });
    if let (true, Some(id)) = (s.offer, &work.work_item) {
        params["ticket_id"] = json!(id);
        params["ticket_title"] = json!(work.name.as_deref().map(work_item_title).unwrap_or_default());
    }
    hw::emit(&json!({ "method": "detail", "params": params }));
    hw::status("running", "Following Azure DevOps");
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for Azure DevOps to be in front");
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

    fn at(path: &str, title: &str) -> Work {
        from_address(&format!("https://dev.azure.com/contoso/Web%20Shop/{path}"), title).unwrap()
    }

    #[test]
    fn areas_and_kinds_of_work() {
        let w = at("_workitems/edit/1234", "Bug 1234: Login fails - Boards - Work - Microsoft Edge");
        assert_eq!(w.describe(), "Bug 1234: Login fails · work item · contoso/Web Shop");
        assert_eq!((w.category, w.work_item.as_deref()), (PLANNING, Some("1234")));
        assert_eq!(work_item_title("Bug 1234: Login fails"), "Login fails");

        let pr = at("_git/web/pullrequest/42", "Pull request 42: Add login - Repos");
        assert_eq!((pr.kind, pr.category, pr.name.as_deref()), ("pull request", REVIEW, Some("Pull request 42: Add login")));
        assert_eq!(at("_git/web?path=/src", "web - Repos").kind, "repository");
        assert_eq!(at("_build/results?buildId=991&view=logs", "Run 991 - Pipelines").kind, "pipeline run");
        assert_eq!(at("_build?definitionId=3", "Nightly - Pipelines").category, CICD);
        assert_eq!(at("_sprints/taskboard/Web/Sprint 12", "Sprint 12 - Boards").kind, "sprint");
        assert_eq!(at("_boards/board/t/Web/Stories?workitem=88", "Stories - Boards").work_item.as_deref(), Some("88"));
        assert_eq!(at("_wiki/wikis/Web.wiki/12/Release-notes", "Release notes - Wiki").category, DOCS);
        assert_eq!(at("_testplans/define?planId=5", "Plan - Test Plans").category, TESTING);
        assert_eq!(at("_settings/repositories", "Settings - Azure DevOps").category, ADMIN);
    }

    #[test]
    fn organisations_and_old_addresses() {
        let old = from_address("https://contoso.visualstudio.com/Web/_git/api/pullrequest/7", "Pull request 7: Fix - Repos").unwrap();
        assert_eq!((old.kind, old.place.as_deref()), ("pull request", Some("contoso/Web")));
        let org = from_address("https://dev.azure.com/contoso/", "Projects - Home - Azure DevOps").unwrap();
        assert_eq!((org.kind, org.category), ("organisation", ADMIN));
        assert_eq!(from_address("https://portal.azure.com/#home", "Home - Microsoft Azure"), None);
        assert_eq!(from_address("https://github.com/contoso/web/pull/1", "Add login - Pull Request"), None);
    }
}

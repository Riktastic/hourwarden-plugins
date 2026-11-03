//! VS Code (and Cursor, Windsurf, VSCodium): the folder of the workspace in
//! front and the git repository it belongs to, such as `contoso/portal`. The
//! folder counts as a customer folder, the repository is matched against
//! customer names and aliases. Remote SSH windows give their server.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell `vscode-git`
//! plugin. The app answers which workspace folders the editors know
//! (capability `vscode.workspaces`) and the git config of a folder's
//! repository (`git.config`); reading the title and the remote is done here.

// Single-threaded wasm: the state lives in statics between calls.
#![allow(static_mut_refs)]

use std::collections::BTreeMap;

use hourwarden_plugin as hw;
use hw::{json, Value};
use regex_lite::Regex;

/// Program names (without .exe) of VS Code and its forks.
const EDITORS: &[&str] = &["code", "code - insiders", "code-insiders", "cursor", "windsurf", "vscodium", "codium"];
/// The workspace list is read again after this long (ms).
const REFRESH_MS: i64 = 5 * 60 * 1000;

struct Patterns {
    app_tail: Regex,
    ssh: Regex,
    separator: Regex,
    remote_tag: Regex,
    workspace_tag: Regex,
}

impl Patterns {
    fn new() -> Patterns {
        Patterns {
            app_tail: Regex::new(r"(?i)\s-\s(Visual Studio Code - Insiders|Visual Studio Code|Cursor|Windsurf|VSCodium)$").unwrap(),
            ssh: Regex::new(r"(?i)\[SSH:\s*([^\]]+)\]").unwrap(),
            separator: Regex::new(r"\s-\s").unwrap(),
            remote_tag: Regex::new(r"(?i)\s*\[(SSH|WSL|Dev Container|Codespaces)[^\]]*\]").unwrap(),
            workspace_tag: Regex::new(r"(?i)\s*\(Workspace\)$").unwrap(),
        }
    }
}

struct State {
    patterns: Option<Patterns>,
    /// Workspace name (lower case) -> folder, and when it was read.
    workspaces: BTreeMap<String, String>,
    read_at: Option<i64>,
    now: i64,
}

static mut STATE: State = State { patterns: None, workspaces: BTreeMap::new(), read_at: None, now: 0 };

/// The workspace named in a title and the SSH server of a remote window:
/// "main.rs - portal - Visual Studio Code" gives "portal".
fn workspace_from_title(p: &Patterns, title: &str, known: &BTreeMap<String, String>) -> (Option<String>, Option<String>) {
    let t = p.app_tail.replace(title, "");
    let t = t.trim_start_matches(['\u{25CF}', ' ']);
    let remote = p.ssh.captures(t).map(|c| c[1].trim().to_string());
    let parts: Vec<String> = p
        .separator
        .split(t)
        .map(|part| {
            let part = p.remote_tag.replace_all(part, "");
            p.workspace_tag.replace(&part, "").trim().to_string()
        })
        .filter(|part| !part.is_empty())
        .collect();
    let name = parts
        .iter()
        .rev()
        .find(|part| known.contains_key(&part.to_lowercase()))
        .or_else(|| if parts.len() >= 2 { parts.get(1) } else { parts.first() })
        .cloned();
    (name, remote)
}

/// The repository in a git config, from its `origin` remote (or any remote):
/// "git@github.com:Contoso/portal.git" gives "Contoso/portal",
/// "https://dev.azure.com/contoso/Web/_git/portal" gives "contoso/Web/portal".
fn git_remote(config: &str) -> Option<String> {
    let section = |name: &str| -> Option<String> {
        let re = Regex::new(&format!(r#"(?ms)^\[remote "{name}"\](.*?)(?:^\[|\z)"#)).ok()?;
        re.captures(config).map(|c| c[1].to_string())
    };
    let body = section("origin").or_else(|| section(r#"[^"]+"#))?;
    let url = Regex::new(r"(?m)^\s*url\s*=\s*(\S+)").unwrap().captures(&body)?[1].to_string();
    let mut path = url;
    for (pattern, with) in [
        (r"(?i)^[a-z+]+://", ""),
        (r"^[^@/]+@", ""),
        (r"^[^/:]+(:\d+)?[/:]", ""),
        (r"(?i)\.git/?$", ""),
        (r"/_git/", "/"),
        (r"(?i)^(scm|v\d+)/", ""),
    ] {
        path = Regex::new(pattern).unwrap().replace(&path, with).into_owned();
    }
    let path = path.trim_matches('/').to_string();
    (!path.is_empty()).then_some(path)
}

fn app_name(exe: &str) -> String {
    let exe = exe.to_lowercase();
    exe.strip_suffix(".exe").unwrap_or(&exe).to_string()
}

fn refresh_workspaces(s: &mut State) -> bool {
    if s.read_at.is_some_and(|at| s.now - at < REFRESH_MS) {
        return true;
    }
    let Some(result) = hw::capability("vscode.workspaces", &Value::Null) else {
        hw::status("degraded", "Could not read the VS Code workspace list");
        return false;
    };
    s.workspaces = result["folders"]
        .as_object()
        .map(|o| o.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect())
        .unwrap_or_default();
    s.read_at = Some(s.now);
    true
}

fn on_foreground(event: &Value) {
    let s = unsafe { &mut STATE };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    if !EDITORS.contains(&app_name(&exe).as_str()) || !refresh_workspaces(s) {
        return;
    }
    let p = s.patterns.get_or_insert_with(Patterns::new);
    let (name, remote) = workspace_from_title(p, event["title"].as_str().unwrap_or(""), &s.workspaces);
    let Some(name) = name else { return };
    let folder = s.workspaces.get(&name.to_lowercase()).cloned();
    let repo = folder
        .as_ref()
        .and_then(|f| hw::capability("git.config", &json!({ "folder": f })))
        .and_then(|r| r["config"].as_str().and_then(git_remote));

    let mut detail = json!({ "hwnd": event["hwnd"], "exe": exe });
    if let Some(folder) = &folder {
        detail["path"] = json!(folder);
    }
    if let Some(repo) = &repo {
        detail["remote_folder"] = json!(repo);
    }
    if let Some(remote) = &remote {
        detail["remote_server"] = json!(remote);
    }
    if folder.is_some() || repo.is_some() || remote.is_some() {
        hw::emit(&json!({ "method": "detail", "params": detail }));
    }
    hw::status("running", &format!("Knows {} VS Code folders", s.workspaces.len()));
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 3000;

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 3) };
    hw::status("running", "Waiting for VS Code to be in front");
}

fn on_event(event: Value) {
    match event["kind"].as_str() {
        Some("tick") => unsafe { STATE.now = event["now"].as_i64().unwrap_or(0) },
        Some("settings") => return unsafe { POLL_MS = hw::poll_ms(&event["settings"], 3) },
        _ => {}
    }
    if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        on_foreground(&fg);
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_to_workspace() {
        let p = Patterns::new();
        let known: BTreeMap<String, String> = [("portal".to_string(), "/home/rik/portal".to_string())].into();
        assert_eq!(workspace_from_title(&p, "\u{25CF} main.rs - portal - Visual Studio Code", &known), (Some("portal".into()), None));
        assert_eq!(
            workspace_from_title(&p, "app.py - api [SSH: web01.contoso.nl] - Cursor", &BTreeMap::new()),
            (Some("api".into()), Some("web01.contoso.nl".into()))
        );
        assert_eq!(workspace_from_title(&p, "Shop (Workspace) - Visual Studio Code", &BTreeMap::new()), (Some("Shop".into()), None));
    }

    #[test]
    fn remotes() {
        assert_eq!(git_remote("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = git@github.com:Contoso/portal.git\n"), Some("Contoso/portal".into()));
        assert_eq!(git_remote("[remote \"origin\"]\n\turl = https://dev.azure.com/contoso/Web/_git/portal\n"), Some("contoso/Web/portal".into()));
        assert_eq!(git_remote("[remote \"upstream\"]\n\turl = ssh://git@gitlab.example.com:2222/team/app.git\n"), Some("team/app".into()));
        assert_eq!(git_remote("[core]\n\tbare = false\n"), None);
    }
}

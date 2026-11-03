//! Saved files in customer folders. Hidden folders, temporary files and
//! bursts (sync clients, git checkouts) are ignored; the app decides whether
//! a change belongs to the work in front.
//!
//! This is the WebAssembly port (plugin API 2) of the PowerShell
//! `file-changes` plugin. The app watches the customer folders (capability
//! `files.changes`, which only covers those folders); what to report is
//! decided here.

// Single-threaded wasm: the state lives in a static between calls.
#![allow(static_mut_refs)]

use std::collections::{BTreeMap, HashMap, VecDeque};

use hourwarden_plugin as hw;
use hw::{json, Value};

const DEFAULT_IGNORE: &[&str] = &["~$*", "*.tmp", "*.temp", "*.lock", "*.crdownload", "*.partial"];
/// A change is held this long; a burst in that time drops it.
const HOLD_MS: i64 = 2000;
/// More than `max_burst` files within this window is a burst ...
const BURST_WINDOW_MS: i64 = 5000;
/// ... and nothing is reported for this long after it.
const QUIET_MS: i64 = 10_000;
/// The same file is reported at most this often.
const REPEAT_MS: i64 = 10_000;
const MAX_RECENT: usize = 500;

#[derive(Default)]
struct State {
    ignore: Vec<String>,
    max_burst: usize,
    burst: VecDeque<i64>,
    quiet_until: i64,
    /// Path -> (first seen, change), in the order seen.
    pending: BTreeMap<String, (i64, String)>,
    recent: HashMap<String, i64>,
    /// The last status reported: (watched, missing).
    shown: Option<(i64, i64)>,
}

static mut STATE: Option<State> = None;

fn state() -> &'static mut State {
    unsafe { STATE.get_or_insert_with(State::default) }
}

fn configure(settings: &Value) {
    let s = state();
    s.ignore = hw::string_list(&settings["ignore_patterns"], DEFAULT_IGNORE);
    s.max_burst = settings["max_burst"].as_u64().unwrap_or(20).clamp(1, 10_000) as usize;
}

/// Matches a file name against a pattern with `*` and `?`, ignoring case.
fn wildcard(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.to_lowercase().chars().collect(), name.to_lowercase().chars().collect());
    let (mut pi, mut ni, mut star, mut mark) = (0, 0, None, 0);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ni;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// Hidden and tool folders, and file names matching an ignore pattern.
fn ignored(path: &str, patterns: &[String]) -> bool {
    let parts: Vec<&str> = path.split(['\\', '/']).collect();
    let (name, folders) = parts.split_last().unwrap_or((&"", &[]));
    if folders.iter().any(|f| f.starts_with('.') || *f == "node_modules") {
        return true;
    }
    patterns.iter().any(|p| wildcard(p, name))
}

/// Counts a new file towards a burst; true while things are quiet after one.
fn in_burst(s: &mut State, now: i64) -> bool {
    s.burst.push_back(now);
    while s.burst.front().is_some_and(|&t| now - t > BURST_WINDOW_MS) {
        s.burst.pop_front();
    }
    if s.burst.len() > s.max_burst {
        s.quiet_until = now + QUIET_MS;
    }
    now < s.quiet_until
}

fn report_status(s: &mut State, watched: i64, missing: i64) {
    if s.shown == Some((watched, missing)) {
        return;
    }
    s.shown = Some((watched, missing));
    if watched == 0 {
        hw::status("degraded", "No customer folders to watch; add folders on the customer pages");
    } else if missing > 0 {
        hw::status("degraded", &format!("Watching {watched} folders; {missing} do not exist"));
    } else {
        hw::status("running", &format!("Watching {watched} customer folders"));
    }
}

/// Takes in new changes and returns the ones to report now.
fn process(s: &mut State, now: i64, changes: &[Value]) -> Vec<(String, String)> {
    for change in changes {
        let Some(path) = change["path"].as_str() else { continue };
        if ignored(path, &s.ignore) || s.pending.contains_key(path) {
            continue;
        }
        // Each file counts once towards a burst (a save gives several events).
        in_burst(s, now);
        s.pending.insert(path.to_string(), (now, change["change"].as_str().unwrap_or("Changed").to_string()));
    }
    if now < s.quiet_until {
        s.pending.clear();
    }
    let ready: Vec<String> = s.pending.iter().filter(|(_, (at, _))| now - at >= HOLD_MS).map(|(p, _)| p.clone()).collect();
    let mut out = Vec::new();
    for path in ready {
        let (_, change) = s.pending.remove(&path).unwrap_or_default();
        if s.recent.get(&path).is_some_and(|&last| now - last < REPEAT_MS) {
            continue;
        }
        s.recent.insert(path.clone(), now);
        out.push((path, change));
    }
    if s.recent.len() > MAX_RECENT {
        s.recent.clear();
    }
    out
}

fn on_tick(now: i64) {
    let Some(answer) = hw::capability("files.changes", &Value::Null) else {
        hw::status("degraded", "The customer folders could not be watched");
        return;
    };
    let s = state();
    report_status(s, answer["watched"].as_i64().unwrap_or(0), answer["missing"].as_i64().unwrap_or(0));
    let changes = answer["changes"].as_array().cloned().unwrap_or_default();
    for (path, change) in process(s, now, &changes) {
        hw::emit(&json!({ "method": "file_changed", "params": { "path": path, "change": change } }));
    }
}

fn on_init(data: Value) {
    configure(&data["settings"]);
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

    fn fresh() -> State {
        State { ignore: DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect(), max_burst: 3, ..State::default() }
    }

    fn change(path: &str) -> Value {
        json!({ "path": path, "change": "Changed" })
    }

    #[test]
    fn patterns_and_folders() {
        assert!(wildcard("~$*", "~$Offerte.docx"));
        assert!(wildcard("*.TMP", "abc.tmp"));
        assert!(!wildcard("*.tmp", "abc.tmpl"));
        let p: Vec<String> = DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect();
        assert!(ignored(r"D:\Klanten\Contoso\.git\index", &p));
        assert!(ignored("/home/rik/app/node_modules/x/y.js", &p));
        assert!(!ignored(r"D:\Klanten\Contoso\Offerte.docx", &p));
    }

    #[test]
    fn held_then_reported_once() {
        let mut s = fresh();
        assert!(process(&mut s, 0, &[change("C:/k/a.docx")]).is_empty());
        assert_eq!(process(&mut s, 2000, &[]), vec![("C:/k/a.docx".to_string(), "Changed".to_string())]);
        // Saved again soon after: not reported again within ten seconds.
        process(&mut s, 3000, &[change("C:/k/a.docx")]);
        assert!(process(&mut s, 5000, &[]).is_empty());
    }

    #[test]
    fn a_burst_drops_everything() {
        let mut s = fresh();
        let many: Vec<Value> = (0..5).map(|i| change(&format!("C:/k/f{i}.txt"))).collect();
        process(&mut s, 0, &many);
        assert!(process(&mut s, 2500, &[]).is_empty());
    }
}

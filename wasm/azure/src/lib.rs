//! Azure and the Microsoft admin centres: what you have open.
//!
//! The Azure portal keeps its location after `#`:
//! `portal.azure.com/#@contoso.onmicrosoft.com/resource/subscriptions/<id>/
//! resourceGroups/rg-web/providers/Microsoft.Web/sites/app-web/configuration`.
//! The provider and type say the kind of resource, the last part what you look
//! at. The admin centres (Entra, Intune, Microsoft 365, Defender, Purview,
//! Exchange, Teams) are recognised by their host. Azure Storage Explorer and
//! Azure Data Studio by program.
//!
//! The plugin reports it as the document of the window, with a kind of work
//! for the activity log's totals. Tenant and subscription IDs are left to the
//! web-addresses plugin.

#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

const DEFAULT_BROWSERS: &[&str] = &["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "vivaldi.exe", "opera.exe", "arc.exe"];

const INFRA: &str = "Infrastructure";
const DATA: &str = "Data";
const SECURITY: &str = "Security and identity";
const MONITORING: &str = "Monitoring";
const INTEGRATION: &str = "Integration";
const AI: &str = "AI";
const COST: &str = "Cost and governance";
const ADMIN: &str = "Administration";

#[derive(Debug, Clone, PartialEq)]
struct Work {
    kind: String,
    category: &'static str,
    name: Option<String>,
    doing: Option<String>,
    group: Option<String>,
}

impl Work {
    fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(n) = self.name.as_deref().filter(|n| !n.is_empty()) {
            parts.push(n.to_string());
        }
        parts.push(self.kind.clone());
        if let Some(d) = &self.doing {
            parts.push(d.clone());
        }
        if let Some(g) = &self.group {
            parts.push(format!("resource group {g}"));
        }
        parts.join(" · ")
    }
}

/// Resource types (provider/type, lower case) with their name and kind of work.
const TYPES: &[(&str, &str, &str)] = &[
    ("microsoft.compute/virtualmachines", "virtual machine", INFRA),
    ("microsoft.compute/virtualmachinescalesets", "VM scale set", INFRA),
    ("microsoft.compute/disks", "disk", INFRA),
    ("microsoft.web/sites", "App Service", INFRA),
    ("microsoft.web/serverfarms", "App Service plan", INFRA),
    ("microsoft.web/staticsites", "Static Web App", INFRA),
    ("microsoft.app/containerapps", "Container App", INFRA),
    ("microsoft.containerservice/managedclusters", "AKS cluster", INFRA),
    ("microsoft.containerregistry/registries", "container registry", INFRA),
    ("microsoft.containerinstance/containergroups", "container instance", INFRA),
    ("microsoft.network/virtualnetworks", "virtual network", INFRA),
    ("microsoft.network/networksecuritygroups", "network security group", INFRA),
    ("microsoft.network/applicationgateways", "application gateway", INFRA),
    ("microsoft.network/loadbalancers", "load balancer", INFRA),
    ("microsoft.network/publicipaddresses", "public IP address", INFRA),
    ("microsoft.network/privateendpoints", "private endpoint", INFRA),
    ("microsoft.network/dnszones", "DNS zone", INFRA),
    ("microsoft.network/privatednszones", "private DNS zone", INFRA),
    ("microsoft.network/frontdoors", "Front Door", INFRA),
    ("microsoft.cdn/profiles", "Front Door / CDN profile", INFRA),
    ("microsoft.network/azurefirewalls", "firewall", SECURITY),
    ("microsoft.network/virtualnetworkgateways", "VPN gateway", INFRA),
    ("microsoft.network/bastionhosts", "Bastion", INFRA),
    ("microsoft.sql/servers/databases", "SQL database", DATA),
    ("microsoft.sql/servers", "SQL server", DATA),
    ("microsoft.sql/managedinstances", "SQL managed instance", DATA),
    ("microsoft.dbforpostgresql/flexibleservers", "PostgreSQL server", DATA),
    ("microsoft.dbformysql/flexibleservers", "MySQL server", DATA),
    ("microsoft.documentdb/databaseaccounts", "Cosmos DB account", DATA),
    ("microsoft.storage/storageaccounts", "storage account", DATA),
    ("microsoft.cache/redis", "Redis cache", DATA),
    ("microsoft.datafactory/factories", "Data Factory", DATA),
    ("microsoft.synapse/workspaces", "Synapse workspace", DATA),
    ("microsoft.databricks/workspaces", "Databricks workspace", DATA),
    ("microsoft.fabric/capacities", "Fabric capacity", DATA),
    ("microsoft.powerbidedicated/capacities", "Power BI capacity", DATA),
    ("microsoft.purview/accounts", "Purview account", DATA),
    ("microsoft.keyvault/vaults", "Key Vault", SECURITY),
    ("microsoft.managedidentity/userassignedidentities", "managed identity", SECURITY),
    ("microsoft.security", "Defender for Cloud", SECURITY),
    ("microsoft.insights/components", "Application Insights", MONITORING),
    ("microsoft.operationalinsights/workspaces", "Log Analytics workspace", MONITORING),
    ("microsoft.insights/actiongroups", "action group", MONITORING),
    ("microsoft.insights/workbooks", "workbook", MONITORING),
    ("microsoft.dashboard/grafana", "Managed Grafana", MONITORING),
    ("microsoft.logic/workflows", "Logic App", INTEGRATION),
    ("microsoft.servicebus/namespaces", "Service Bus", INTEGRATION),
    ("microsoft.eventhub/namespaces", "Event Hubs", INTEGRATION),
    ("microsoft.eventgrid/topics", "Event Grid topic", INTEGRATION),
    ("microsoft.apimanagement/service", "API Management", INTEGRATION),
    ("microsoft.cognitiveservices/accounts", "Azure AI service", AI),
    ("microsoft.machinelearningservices/workspaces", "Azure Machine Learning", AI),
    ("microsoft.search/searchservices", "AI Search", AI),
    ("microsoft.botservice/botservices", "bot", AI),
    ("microsoft.automation/automationaccounts", "Automation account", ADMIN),
    ("microsoft.recoveryservices/vaults", "Recovery Services vault", INFRA),
    ("microsoft.costmanagement", "Cost Management", COST),
    ("microsoft.consumption", "Cost Management", COST),
    ("microsoft.authorization/policyassignments", "policy", COST),
    ("microsoft.resources/deployments", "deployment", INFRA),
];

/// Blades: the last part of a resource's location, for people.
const BLADES: &[(&str, &str)] = &[
    ("overview", "overview"),
    ("configuration", "configuration"),
    ("environmentvariablesappsettings", "configuration"),
    ("appsettings", "configuration"),
    ("logs", "logs"),
    ("logstream", "log stream"),
    ("metrics", "metrics"),
    ("alerts", "alerts"),
    ("users", "access control"),
    ("iam", "access control"),
    ("accesscontrol", "access control"),
    ("networking", "networking"),
    ("deploymentcenter", "deployment"),
    ("deployments", "deployment"),
    ("diagnose", "diagnosing"),
    ("troubleshoot", "diagnosing"),
    ("secrets", "secrets"),
    ("keys", "keys"),
    ("certificates", "certificates"),
    ("accesspolicies", "access policies"),
    ("containers", "containers"),
    ("storagebrowser", "storage browser"),
    ("queryeditor", "query editor"),
    ("costanalysis", "cost analysis"),
    ("properties", "properties"),
    ("scale", "scaling"),
    ("scaleout", "scaling"),
    ("scaleup", "scaling"),
    ("backup", "backup"),
    ("connect", "connecting"),
    ("activitylog", "activity log"),
];

/// Portal pages outside one resource: `#view/…`, `#browse/…`, `#blade/…`.
const PAGES: &[(&str, &str, &str)] = &[
    ("costmanagement", "Cost Management", COST),
    ("billing", "billing", COST),
    ("policy", "Azure Policy", COST),
    ("advisor", "Advisor", COST),
    ("aad", "Entra ID", SECURITY),
    ("activedirectory", "Entra ID", SECURITY),
    ("security", "Defender for Cloud", SECURITY),
    ("monitor", "Azure Monitor", MONITORING),
    ("loganalytics", "Log Analytics", MONITORING),
    ("subscriptions", "subscriptions", ADMIN),
    ("resourcegroups", "resource groups", ADMIN),
    ("managementgroups", "management groups", COST),
    ("support", "support", ADMIN),
    ("quickstart", "quickstart", ADMIN),
    ("marketplace", "Marketplace", INFRA),
];

/// The admin centres, by host.
const CENTRES: &[(&str, &str, &str)] = &[
    ("entra.microsoft.com", "Entra admin centre", SECURITY),
    ("aad.portal.azure.com", "Entra admin centre", SECURITY),
    ("intune.microsoft.com", "Intune", ADMIN),
    ("endpoint.microsoft.com", "Intune", ADMIN),
    ("admin.microsoft.com", "Microsoft 365 admin centre", ADMIN),
    ("security.microsoft.com", "Defender", SECURITY),
    ("purview.microsoft.com", "Purview", SECURITY),
    ("compliance.microsoft.com", "Purview", SECURITY),
    ("admin.exchange.microsoft.com", "Exchange admin centre", ADMIN),
    ("admin.teams.microsoft.com", "Teams admin centre", ADMIN),
    ("admin.powerplatform.microsoft.com", "Power Platform admin centre", ADMIN),
    ("make.powerapps.com", "Power Apps", INTEGRATION),
    ("make.powerautomate.com", "Power Automate", INTEGRATION),
    ("ai.azure.com", "Azure AI Foundry", AI),
    ("oai.azure.com", "Azure OpenAI Studio", AI),
    ("ml.azure.com", "Azure Machine Learning studio", AI),
    ("adf.azure.com", "Data Factory studio", DATA),
    ("web.azuresynapse.net", "Synapse Studio", DATA),
    ("shell.azure.com", "Cloud Shell", INFRA),
];

fn is_portal(host: &str) -> bool {
    matches!(host, "portal.azure.com" | "ms.portal.azure.com" | "preview.portal.azure.com" | "portal.azure.us" | "portal.azure.cn")
}

/// "Microsoft.Web/sites" style type from a resource path's segments.
fn resource(seg: &[String]) -> Option<Work> {
    let lower: Vec<String> = seg.iter().map(|s| s.to_lowercase()).collect();
    let group = lower.iter().position(|s| s == "resourcegroups").and_then(|i| seg.get(i + 1)).cloned();
    let Some(p) = lower.iter().position(|s| s == "providers") else {
        // A resource group or subscription itself.
        return group.clone().map(|g| Work { kind: "resource group".into(), category: ADMIN, name: Some(g), doing: blade(&lower[lower.len().saturating_sub(1)..]), group: None });
    };
    let provider = lower.get(p + 1)?;
    // Type and name alternate after the provider: sites/app-web[/slots/staging]/configuration.
    let after = &seg[p + 2..];
    let mut types = Vec::new();
    let mut name = None;
    let mut i = 0;
    while i + 1 < after.len() {
        types.push(after[i].to_lowercase());
        name = Some(after[i + 1].clone());
        i += 2;
    }
    let rest: Vec<String> = after[i..].iter().map(|s| s.to_lowercase()).collect();
    let full = format!("{provider}/{}", types.join("/"));
    let first = format!("{provider}/{}", types.first().cloned().unwrap_or_default());
    let (kind, category) = TYPES
        .iter()
        .find(|(t, _, _)| *t == full)
        .or_else(|| TYPES.iter().find(|(t, _, _)| *t == first))
        .or_else(|| TYPES.iter().find(|(t, _, _)| *t == provider))
        .map(|(_, k, c)| (k.to_string(), *c))
        .unwrap_or_else(|| (types.last().cloned().unwrap_or_else(|| provider.clone()), INFRA));
    Some(Work { kind, category, name, doing: blade(&rest), group })
}

fn blade(rest: &[String]) -> Option<String> {
    let last = rest.iter().rev().find(|s| !s.is_empty())?;
    BLADES.iter().find(|(b, _)| *b == last).map(|(_, d)| d.to_string())
}

/// What an Azure portal or admin centre address says you work on.
fn from_address(address: &str, title: &str) -> Option<Work> {
    let a = address.trim();
    let a = a.split_once("://").map_or(a, |(_, r)| r);
    let host = a.split(['/', '#', '?']).next().unwrap_or("").to_lowercase();
    if let Some((_, name, category)) = CENTRES.iter().find(|(h, _, _)| *h == host) {
        let page = title.split(" - ").next().map(str::trim).filter(|t| !t.is_empty() && !t.eq_ignore_ascii_case(name)).map(str::to_string);
        return Some(Work { kind: name.to_string(), category, name: page, doing: None, group: None });
    }
    if !is_portal(&host) {
        return None;
    }
    let fragment = a.split_once('#').map(|(_, f)| f).unwrap_or("");
    let seg: Vec<String> = fragment.split('/').filter(|s| !s.is_empty() && !s.starts_with('@')).map(|s| s.replace("%20", " ")).collect();
    let lower: Vec<String> = seg.iter().map(|s| s.to_lowercase()).collect();
    if lower.iter().any(|s| s == "resourcegroups" || s == "providers") {
        return resource(&seg);
    }
    // #view/Microsoft_Azure_CostManagement/…, #browse/resourcegroups, #home
    let joined = lower.join("/");
    if let Some((_, name, category)) = PAGES.iter().find(|(k, _, _)| joined.contains(k)) {
        return Some(Work { kind: name.to_string(), category, name: None, doing: None, group: None });
    }
    Some(Work { kind: "Azure portal".into(), category: ADMIN, name: None, doing: None, group: None })
}

/// Azure's desktop tools, by program.
fn from_desktop(exe: &str, title: &str) -> Option<Work> {
    let name = title.split(" - ").next().map(str::trim).filter(|t| !t.is_empty()).map(str::to_string);
    match hw::app_name(exe).as_str() {
        "storageexplorer" => Some(Work { kind: "Azure Storage Explorer".into(), category: DATA, name: None, doing: None, group: None }),
        "azuredatastudio" => Some(Work { kind: "Azure Data Studio".into(), category: DATA, name: name.filter(|n| n != "Azure Data Studio"), doing: None, group: None }),
        _ => None,
    }
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

/// Only these titles are worth reading the address for.
fn looks_like_azure(title: &str) -> bool {
    let t = title.to_lowercase();
    ["microsoft azure", "azure", "entra", "intune", "microsoft 365 admin", "defender", "purview", "exchange admin", "teams admin", "power platform", "power apps", "power automate", "ai foundry"]
        .iter()
        .any(|w| t.contains(w))
}

fn on_foreground(event: &Value) {
    let s = unsafe { &mut STATE };
    let exe = event["exe"].as_str().unwrap_or("").to_lowercase();
    let title = event["title"].as_str().unwrap_or("");
    let work = if s.browsers.contains(&hw::app_name(&exe)) {
        if !looks_like_azure(title) {
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
    hw::emit(&json!({ "method": "detail", "params": { "hwnd": event["hwnd"], "exe": exe, "document": document, "category": work.category } }));
    hw::status("running", "Following Azure");
}

fn on_init(data: Value) {
    configure(&data["settings"]);
    hw::status("running", "Waiting for Azure to be in front");
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

    const SUB: &str = "11111111-2222-3333-4444-555555555555";

    fn portal(fragment: &str) -> Work {
        from_address(&format!("https://portal.azure.com/#@contoso.onmicrosoft.com/{fragment}"), "Microsoft Azure").unwrap()
    }

    #[test]
    fn resources_their_group_and_blade() {
        let app = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg-web/providers/Microsoft.Web/sites/app-web/configuration"));
        assert_eq!(app.describe(), "app-web · App Service · configuration · resource group rg-web");
        assert_eq!(app.category, INFRA);

        let db = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg-data/providers/Microsoft.Sql/servers/sql-1/databases/sales/queryEditor"));
        assert_eq!((db.kind.as_str(), db.name.as_deref(), db.doing.as_deref(), db.category), ("SQL database", Some("sales"), Some("query editor"), DATA));

        let kv = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg-sec/providers/Microsoft.KeyVault/vaults/kv-prod/secrets"));
        assert_eq!((kv.kind.as_str(), kv.doing.as_deref(), kv.category), ("Key Vault", Some("secrets"), SECURITY));

        let law = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg-mon/providers/Microsoft.OperationalInsights/workspaces/law-1/logs"));
        assert_eq!((law.kind.as_str(), law.category), ("Log Analytics workspace", MONITORING));

        let unknown = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg/providers/Microsoft.Contoso/widgets/w1/overview"));
        assert_eq!((unknown.kind.as_str(), unknown.doing.as_deref()), ("widgets", Some("overview")));

        let rg = portal(&format!("resource/subscriptions/{SUB}/resourceGroups/rg-web/overview"));
        assert_eq!((rg.kind.as_str(), rg.name.as_deref(), rg.category), ("resource group", Some("rg-web"), ADMIN));
    }

    #[test]
    fn portal_pages_and_admin_centres() {
        assert_eq!(portal("view/Microsoft_Azure_CostManagement/Menu/~/costanalysis").category, COST);
        assert_eq!(portal("view/Microsoft_AAD_IAM/ActiveDirectoryMenuBlade/~/Overview").kind, "Entra ID");
        assert_eq!(portal("home").kind, "Azure portal");
        let intune = from_address("https://intune.microsoft.com/#view/Microsoft_Intune_DeviceSettings/DevicesMenu/~/overview", "Devices - Microsoft Intune admin center").unwrap();
        assert_eq!((intune.describe().as_str(), intune.category), ("Devices · Intune", ADMIN));
        assert_eq!(from_address("https://security.microsoft.com/incidents", "Incidents - Microsoft Defender").unwrap().category, SECURITY);
        assert_eq!(from_address("https://dev.azure.com/contoso/web", "Overview"), None, "that is DevOps");
        assert_eq!(from_address("https://www.bing.com/search?q=azure", "azure - Search"), None);
    }

    #[test]
    fn desktop_tools() {
        assert_eq!(from_desktop("StorageExplorer.exe", "Microsoft Azure Storage Explorer").unwrap().category, DATA);
        assert_eq!(from_desktop("azuredatastudio.exe", "sales.sql - Azure Data Studio").unwrap().name.as_deref(), Some("sales.sql"));
        assert_eq!(from_desktop("code.exe", "main.rs - VS Code"), None);
    }
}

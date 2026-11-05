# Microsoft Fabric and Power BI

**Works on:** Windows only: it reads the browser's address bar through Windows UI Automation.

Recognises what you do in Microsoft Fabric and Power BI, so the activity log and your entries say more than "Power BI": which item you have open, and whether you are editing a semantic model.

## What it recognises

In Edge, Chrome, Firefox, Brave, Vivaldi, Opera and Arc, on Power BI and Fabric pages (`app.powerbi.com`, `app.fabric.microsoft.com`):

| You have open | Shown as |
|---|---|
| A report | *Sales overview · report* |
| A semantic model's page | *Sales · semantic model · details* |
| A semantic model in the web model view | *Sales · semantic model · editing the model* |
| A notebook, lakehouse, warehouse, pipeline, dataflow, eventstream, KQL database… | *Load customers · notebook* |
| A workspace | *Finance · workspace* |
| The monitoring hub, admin portal, OneLake catalog, deployment pipelines | *monitoring hub* |

The name comes from the browser tab; the kind of item and the view from the address.

On the desktop:

| Program | Shown as |
|---|---|
| Power BI Desktop | *Sales · Power BI Desktop file* |
| Tabular Editor 2 and 3 | *workspace Finance · semantic model · editing in Tabular Editor* |
| DAX Studio | *workspace Finance · DAX queries · in DAX Studio* |
| SQL Server Management Studio on an XMLA endpoint (`powerbi://…/myorg/Finance`) | *workspace Finance · semantic model · XMLA in SSMS* |

## Kinds of work

Each item also gets a kind of work. The activity log adds up the time per kind, so an entry or a day shows, for example, *Data modelling 2:10 · Reporting 1:05 · Data engineering 0:40*:

| Kind of work | Items |
|---|---|
| Reporting | reports, paginated reports, dashboards, scorecards, apps, Power BI Desktop files |
| Data modelling | semantic models (also in Tabular Editor and over XMLA), DAX Studio |
| Data engineering | notebooks, lakehouses, Spark job definitions, environments, GraphQL APIs |
| Data integration | data pipelines, dataflows, copy jobs, mirrored databases |
| Data warehousing | warehouses, SQL analytics endpoints, SQL databases |
| Real-time analytics | eventhouses, eventstreams, KQL databases and querysets, real-time dashboards, Activator |
| Data science | ML models and experiments |
| Administration | workspaces, admin portal, capacity settings, monitoring hub, deployment pipelines, OneLake catalog, variable libraries |

## Customer and tenant

This plugin says *what* you do. Which customer it is for comes from the rest of HourWarden: add the *web-addresses* plugin, which reports the Microsoft tenant and workspace IDs of Fabric and Power BI addresses, and add those IDs to the customer's page.

## A page it does not know

With *Note pages it does not know* on (off by default), when the plugin falls back to a general description, its log (Plugins → Show log) says *Not recognised yet* with the shape of the address: IDs and numbers as `{id}`, without query values. Please report that line, so the page can be added. Names in the address (a resource group, a project) can show in it; the log stays in memory and is never sent anywhere.

## What it reads

The title of the window in front, and the address bar of the browser only while a Power BI or Fabric page is in front. Nothing leaves the computer.

## Limits

- Microsoft changes Fabric's addresses now and then; an unknown kind of item shows as *workspace item*. Please report it, with the address (IDs may be replaced by x).
- Editing a report in the browser does not always change its address, so editing and reading a report can look the same.
- Power BI Desktop does not say in its title whether you are in report or model view.

## Settings

- **Browsers**: the programs whose address bar is read.
- **Check every**: how often, in seconds.

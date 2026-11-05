# Azure and Microsoft admin centres

**Works on:** Windows only: it reads the browser's address bar through Windows UI Automation.

Recognises what you do in the Azure portal and the Microsoft admin centres, so the activity log and your entries say which resource or admin centre the time went to.

## The Azure portal

On `portal.azure.com` the address says which resource you have open, in which resource group, and what you look at:

| You have open | Shown as |
|---|---|
| An App Service's configuration | *app-web · App Service · configuration · resource group rg-web* |
| A SQL database in the query editor | *sales · SQL database · query editor · resource group rg-data* |
| A Key Vault's secrets | *kv-prod · Key Vault · secrets · resource group rg-sec* |
| A resource group | *rg-web · resource group · overview* |
| Cost Management, Policy, Advisor, Entra ID, Defender for Cloud, Monitor | *Cost Management* |

About 60 resource types are known by name (virtual machines, App Service, Container Apps, AKS, networks, SQL, Cosmos DB, storage, Data Factory, Synapse, Databricks, Key Vault, Application Insights, Log Analytics, Logic Apps, Service Bus, API Management, Azure AI services and more). Another type is shown by its own name from the address. What you look at (overview, configuration, logs, metrics, access control, networking, deployment, secrets, scaling, backup…) is added when it is known.

## Inside Entra, storage accounts, Data Factory and Databricks

| You have open | Shown as |
|---|---|
| Entra: users, groups, app registrations, enterprise applications, conditional access, roles, PIM, identity governance, sign-in and audit logs, devices, authentication methods | *Entra · conditional access* |
| A storage account's containers, file shares, queues, tables, lifecycle management, access keys or shared access signatures | *stsales · storage account · file shares · resource group rg-data* |
| Data Factory studio (`adf.azure.com`), authoring a pipeline, dataset, data flow, trigger or linked service | *Load sales · pipeline · authoring · factory adf-sales* |
| Data Factory studio, monitoring pipeline or trigger runs, or managing linked services and integration runtimes | *adf-sales · pipeline runs · monitoring* |
| Databricks (`*.azuredatabricks.net`): a notebook, job or job run, pipeline, SQL query, dashboard, ML experiment, Genie space or compute | *Load bronze · notebook* |
| Databricks Catalog Explorer: a catalog, schema or table | *main.sales.orders · table* |

Entra counts as security and identity; Data Factory and Databricks as data.

## Admin centres and studios

Entra, Intune, the Microsoft 365, Exchange, Teams and Power Platform admin centres, Defender, Purview, Power Apps, Power Automate, Azure AI Foundry, Azure OpenAI Studio, Azure Machine Learning studio, Data Factory studio, Synapse Studio and Cloud Shell: *Devices · Intune*, with the page from the tab title.

On the desktop: Azure Storage Explorer and Azure Data Studio.

## Kinds of work

| Kind of work | For example |
|---|---|
| Infrastructure | virtual machines, App Service, containers, networks, Cloud Shell |
| Data | SQL, Cosmos DB, storage, Data Factory, Synapse, Databricks, Fabric capacities, Storage Explorer |
| Security and identity | Entra, Key Vault, managed identities, firewalls, Defender, Purview |
| Monitoring | Application Insights, Log Analytics, Azure Monitor, workbooks, Grafana |
| Integration | Logic Apps, Service Bus, Event Hubs, API Management, Power Apps, Power Automate |
| AI | Azure AI services and Foundry, Azure OpenAI, Machine Learning, AI Search |
| Cost and governance | Cost Management, billing, Policy, Advisor, management groups |
| Administration | resource groups, subscriptions, Intune, the Microsoft 365, Exchange and Teams admin centres |

The activity log adds up the time per kind of work.

## Customer and tenant

This plugin says *what* you do. The tenant and subscription come from the *web-addresses* plugin; add them to the customer's page and the time goes to the right customer.

## A page it does not know

With *Note pages it does not know* on (off by default), when the plugin falls back to a general description, its log (Plugins → Show log) says *Not recognised yet* with the shape of the address: IDs and numbers as `{id}`, without query values. Please report that line, so the page can be added. Names in the address (a resource group, a project) can show in it; the log stays in memory and is never sent anywhere.

## What it reads

The title of the window in front, and the address bar of the browser only while an Azure or Microsoft admin page is in front. Nothing leaves the computer.

## Settings

- **Browsers**: the programs whose address bar is read.
- **Check every**: how often, in seconds.

# Web addresses in the browser

**Works on:** Windows only: it reads the address bar through Windows UI Automation. Linux has no equivalent that works for every browser.

Reads the domain of the page in front in Edge, Chrome, Brave and other Chromium browsers, such as portal.contoso.com. It counts like a mail domain: add a customer's web domains (contoso.com, contoso.sharepoint.com) on the customer page. Azure portal, Entra, Power BI and Fabric addresses also give the customer's Microsoft tenant (contoso.onmicrosoft.com or its tenant ID), Azure subscription IDs and Power BI or Fabric workspace IDs; add these to the customer page too. Each ID is listed in this plugin's log the first time it is seen. SharePoint admin and OneDrive addresses count as the customer's SharePoint. Useful when you use one browser profile for several customers.

## What it does

Reads the address of the page in front in Edge, Chrome, Firefox, Brave, Vivaldi, Opera and Arc, and sends Timekeeper:

- the **domain** (`portal.contoso.com`); SharePoint admin and OneDrive addresses count as the customer's SharePoint (`contoso.sharepoint.com`);
- the customer's **Microsoft tenant**: from `#@contoso.onmicrosoft.com` (or `#@` with a tenant ID) in Azure portal and Entra addresses, and from `ctid=` in Power BI and Fabric addresses;
- **Azure subscription IDs** from `/subscriptions/...` in Azure portal addresses;
- **Power BI and Fabric workspace IDs** from `/groups/...`.

Never the full address.

## Setting it up

On the customer page, under *Domains and Microsoft IDs*, add:

- the customer's web domains: `contoso.com`, `contoso.sharepoint.com`;
- its tenant: `contoso.onmicrosoft.com` or the tenant ID;
- subscription and workspace IDs, when several customers' work happens in one tenant.

To find the IDs, open the customer's report or portal page: the plugin writes each new ID to its log once (for example `Seen tenant 72f988bf-... on app.powerbi.com`). Copy it from the log under *Plugins*.

Add your own company's sites under *Ignore these domains*.

## Useful when

You use one browser profile for several customers. With a browser profile per customer, an Edge profile rule on the customer page is simpler and stronger.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Browsers | `msedge.exe, chrome.exe, firefox.exe, brave.exe, vivaldi.exe...` | Programs whose address bar is read. |
| Ignore these domains | `(none)` | Domains (and their subdomains) that are never reported, such as your own company's intranet. |
| Check every | `3` | How often the address bar is read, in seconds. |

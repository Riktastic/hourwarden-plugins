# Ticket IDs from window titles

**Works on:** Windows and Linux. On Linux, plugins need PowerShell 7 (`pwsh`).

Spots ticket numbers such as CON-123 (Jira), I 2410 123 (TopDesk) or INC0012345 (ServiceNow) in the title of your browser, Outlook or Teams, and offers them in the top bar as the task. It never changes the customer and does not connect to Jira or TopDesk.

## What it does

Spots ticket numbers in the title of your browser, Outlook or Teams:

| System | Example |
|---|---|
| Jira and similar | `CON-123` |
| TopDesk | `I 2410 123` |
| ServiceNow | `INC0012345`, `RITM0012345` |

The top bar offers the ticket as the task: **Use as task** starts the open task with that ticket ID for the current customer from your task list, or adds a new one. The plugin never changes the customer and does not connect to Jira, TopDesk or ServiceNow.

## Setting it up

Nothing to do for the usual systems. Add your own patterns (regular expressions) under *Ticket patterns*, and words that look like tickets but are not under *Not a ticket*.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Applications | `msedge.exe, chrome.exe, firefox.exe, brave.exe, outlook.e...` | Programs whose window titles are read. |
| Ticket patterns | `\b[A-Z][A-Z0-9]{1,9}-\d{1,7}\b, \b[IMWC] ?\d{4}[ -]\d{3,6...` | Regular expressions for ticket IDs, tried in this order. The defaults cover Jira (ABC-123), TopDesk (I 2410 123) and ServiceNow (INC0012345). |
| Not a ticket | `^(UTF\|ISO\|SHA\|MD\|COVID\|WIN\|TLS\|SSL\|IPV\|HTTP\|RFC...` | Regular expression for matches that are not tickets, such as UTF-8 or COVID-19. |
| Check every | `2` | How often the window title is checked, in seconds. |

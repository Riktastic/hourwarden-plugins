# Azure DevOps

**Works on:** Windows only: it reads the browser's address bar through Windows UI Automation.

Recognises what you do in Azure DevOps, so the activity log and your entries say which work item, pull request or pipeline the time went to.

## What it recognises

In Edge, Chrome, Firefox, Brave, Vivaldi, Opera and Arc, on `dev.azure.com` and `<organisation>.visualstudio.com`:

| You have open | Shown as | Kind of work |
|---|---|---|
| A work item | *Bug 1234: Login fails · work item · contoso/Web Shop* | Planning |
| A board, backlog, sprint, query, dashboard or delivery plan | *Stories · board · contoso/Web Shop* | Planning |
| A pull request, or the list of them | *Pull request 42: Add login · pull request · contoso/Web Shop* | Code review |
| A repository, its commits or branches, code search, a new pull request | *web · repository · contoso/Web Shop* | Development |
| A pipeline, a pipeline run, a release, an environment, the library, artifacts | *Run 991 · pipeline run · contoso/Web Shop* | CI/CD |
| A test plan | *Plan · test plan · contoso/Web Shop* | Testing |
| A wiki page | *Release notes · wiki page · contoso/Web Shop* | Documentation |
| Project or organisation settings | *settings · contoso/Web Shop* | Administration |

The name comes from the browser tab; the area, organisation and project from the address. The activity log adds up the time per kind of work.

## The work item as the task

When a work item is open (also one opened from a board), the top bar offers its number and title as the task, like a ticket ID: *1234 Login fails*. Turn it off with *Offer the work item as the task*.

## Customer

This plugin says *what* you do. Which customer it is for comes from the rest of HourWarden, for example a rule or the customer's name in the organisation or project.

## A page it does not know

With *Note pages it does not know* on (off by default), when the plugin falls back to a general description, its log (Plugins → Show log) says *Not recognised yet* with the shape of the address: IDs and numbers as `{id}`, without query values. Please report that line, so the page can be added. Names in the address (a resource group, a project) can show in it; the log stays in memory and is never sent anywhere.

## What it reads

The title of the window in front, and the address bar of the browser only while an Azure DevOps page is in front. Nothing leaves the computer.

## Settings

- **Browsers**: the programs whose address bar is read.
- **Offer the work item as the task**: on by default.
- **Check every**: how often, in seconds.

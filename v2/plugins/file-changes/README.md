# Saved files in customer folders

**Works on:** Windows and Linux.

Notices files you save, create or rename in customer folders. When the app in front does not show which file you work on, a file just saved in a customer folder decides the customer.

## What it does

Watches your customer folders for files you save, create or rename. Some programs do not show which file you are working on (a drawing tool, a script run from a terminal); a file just saved in a customer folder then decides the customer.

Large bursts (more than 20 files within five seconds) are a sync client or a git checkout, not you, and are ignored.

## Setting it up

Add customer folders on the customer pages; the plugin watches exactly those folders.

## What it reads

Only file names in the customer folders, never their content.

## When it does not work

- Folders on network shares do not always report changes; local and synced folders do.
- Temporary files (`~$*`, `*.tmp`) are ignored; add your own patterns under *Ignore files matching*.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Ignore bursts of more than | `20` | More changed files than this within five seconds is a sync or a checkout, not you; such bursts are ignored. |
| Ignore files matching | `~$*, *.tmp, *.temp, *.lock, *.crdownload, *.partial` | File name patterns that are never reported, such as temporary files. |

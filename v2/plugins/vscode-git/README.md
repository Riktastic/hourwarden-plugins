# VS Code folders and git repositories

**Works on:** Windows and Linux.

Finds the folder of the VS Code (or Cursor, Windsurf, VSCodium) window in front and the git repository it belongs to, such as contoso/portal. The folder counts as a customer folder; the repository's owner and name are matched against customer names and aliases. Remote SSH windows give their server.

## What it does

Finds the folder of the VS Code (or Cursor, Windsurf, VSCodium) window in front and the git repository it belongs to:

- the folder counts as a customer folder;
- the repository's owner and name (such as `contoso/portal`) are matched against customer names and aliases;
- Remote SSH windows give their server, as the SSH plugin does.

## Setting it up

Keep customer code in customer folders, or give customers an alias that matches their repository owner or name.

## What it reads

The title of VS Code windows, VS Code's list of recent workspace folders, and the `.git/config` of the open folder.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `3` | How often the window in front is checked, in seconds. |

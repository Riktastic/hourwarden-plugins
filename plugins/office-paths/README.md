# Office and Explorer paths

Gives the full path of the document open in Word, Excel and PowerPoint, and of the folder open in File Explorer, so customer folders are recognised even when the title only shows the file name.

## What it does

Word, Excel and PowerPoint often show only the file name in their title, and File Explorer only the folder name. This plugin asks them for the full path, so a document in a customer folder is recognised even when its name says nothing.

## Setting it up

Nothing to do besides adding customer folders on the customer pages.

## What it reads

The path of the active document and of the folder open in Explorer, never their content.

## When it does not work

- Office must be the desktop version (not Office on the web).
- Documents opened from SharePoint or OneDrive have a web address as their path; the web addresses plugin and domains on the customer page cover those.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `3` | How often the active document is checked while an Office app or Explorer is in front, in seconds. |

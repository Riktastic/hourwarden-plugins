# SSH sessions in terminals

**Works on:** Windows and Linux.

Recognises the server of an SSH session from the title of Windows Terminal, PuTTY, KiTTY, SecureCRT, MobaXterm and other terminals, so time on a customer's server goes to that customer, like a remote desktop session.

## What it does

Recognises the server of an SSH session from the title of Windows Terminal, PuTTY, KiTTY, SecureCRT, MobaXterm, Termius and other terminals. Time on a customer's server goes to that customer, as for a remote desktop session.

## Setting it up

- Add a **remote server** rule with the server name on the customer page, or an alias that appears in it.
- Add your own computers and WSL distributions under *Your own machines*, so their prompts are not taken for servers.

## What it reads

Only the titles of terminal windows. Your shell should show `user@host` in the title (most do by default).

## Settings

| Setting | Default | What it does |
|---|---|---|
| Terminals | `windowsterminal.exe, openconsole.exe, conhost.exe, putty....` | Programs whose window titles are read. |
| Your own machines | `(none)` | Names of your own computers and WSL distributions, whose prompts are not remote servers. This computer is always left out. |
| Check every | `2` | How often the window title is checked, in seconds. |

# Servers in admin tools

Recognises the server, database or domain you manage from the title of SQL Server Management Studio, WinSCP, FileZilla, Hyper-V and VMware consoles, and Windows management consoles such as Active Directory and DNS. Time there goes to the customer whose server or domain it is.

## What it does

Reads the title of the admin tool in front and takes the server, database or domain out of it:

| Program | What it finds |
|---|---|
| SQL Server Management Studio | the server (and database) you are connected to |
| WinSCP, FileZilla | the server of the session |
| Hyper-V Manager, Virtual Machine Connection | the host and virtual machine |
| VMware Remote Console | the virtual machine |
| Management consoles (Active Directory, DNS, Group Policy) | the domain or domain controller |

## Setting it up

Tell Timekeeper whose servers they are, on the customer page:

- add a **remote server** rule with the server name (for example `sql01.contoso.local`), or
- add an **alias** that appears in the server or domain name (for example `contoso`).

## What it reads

Only the title of the window in front. It does not connect to the servers.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `2` | How often the window title is checked, in seconds. |

# VS Code (and Cursor, Windsurf, VSCodium): the folder of the workspace in
# front and the git repository it belongs to.
#
# The title shows the workspace name ("main.rs - Timekeeper - Visual Studio
# Code"). VS Code's own workspace list maps that name to its folder, and
# .git/config gives the repository ("contoso/portal"). The folder is
# matched like a customer folder, the repository like a customer name.

. "$PSScriptRoot\protocol.ps1"

$editors = [ordered]@{
    'code.exe'            = 'Code'
    'code - insiders.exe' = 'Code - Insiders'
    'cursor.exe'          = 'Cursor'
    'windsurf.exe'        = 'Windsurf'
    'vscodium.exe'        = 'VSCodium'
}
$appNames = 'Visual Studio Code - Insiders|Visual Studio Code|Cursor|Windsurf|VSCodium'
$script:workspaces = @{}
$script:workspacesAt = [DateTime]::MinValue

# "file:///d%3A/Projects/Timekeeper" -> "d:\Projects\Timekeeper"; remote
# workspaces (vscode-remote://) have no local folder.
function ConvertFrom-FileUri([string]$Uri) {
    if (-not $Uri -or -not $Uri.StartsWith('file:///')) { return $null }
    $path = [Uri]::UnescapeDataString($Uri.Substring(8)).Replace('/', '\')
    return $path.TrimEnd('\')
}

# Workspace name -> folder, from VS Code's workspaceStorage; the most
# recently used folder wins when two have the same name.
function Get-WorkspaceFolders([string[]]$StorageDirs) {
    $map = @{}
    $files = foreach ($dir in $StorageDirs) { Get-ChildItem $dir -Filter workspace.json -Recurse -Depth 1 -ErrorAction SilentlyContinue }
    foreach ($file in @($files | Sort-Object LastWriteTime)) {
        try {
            $folder = ConvertFrom-FileUri ((Get-Content $file.FullName -Raw | ConvertFrom-Json).folder)
            if ($folder) { $map[(Split-Path $folder -Leaf).ToLower()] = $folder }
        } catch { }
    }
    return $map
}

# The workspace in a title: the part that is a known workspace, else the
# part before the application name. Also the SSH host of a remote window.
function Get-WorkspaceFromTitle([string]$Title, [string[]]$Known = @()) {
    $t = ($Title -replace "\s-\s($appNames)$", '').TrimStart([char]0x25CF, ' ')
    $remote = $null
    if ($t -match '\[SSH:\s*([^\]]+)\]') { $remote = $Matches[1].Trim() }
    $parts = @($t -split '\s-\s' | ForEach-Object { ($_ -replace '\s*\[(SSH|WSL|Dev Container|Codespaces)[^\]]*\]', '' -replace '\s*\(Workspace\)$', '').Trim() } | Where-Object { $_ })
    $name = $parts | Where-Object { $Known -contains $_.ToLower() } | Select-Object -Last 1
    if (-not $name) { $name = if ($parts.Count -ge 2) { $parts[1] } else { $parts | Select-Object -First 1 } }
    return @{ name = $name; remote_server = $remote }
}

# "git@github.com:Contoso/portal.git", "https://dev.azure.com/contoso/Web/_git/portal"
# -> "Contoso/portal", "contoso/Web/portal".
function Get-GitRemote([string]$Config) {
    if (-not $Config) { return $null }
    $section = [regex]::Match($Config, '(?ms)^\[remote "origin"\](.*?)(^\[|\z)').Groups[1].Value
    if (-not $section) { $section = [regex]::Match($Config, '(?ms)^\[remote "[^"]+"\](.*?)(^\[|\z)').Groups[1].Value }
    $url = [regex]::Match($section, '(?m)^\s*url\s*=\s*(\S+)').Groups[1].Value
    if (-not $url) { return $null }
    $path = $url -replace '^[a-z+]+://', '' -replace '^[^@/]+@', '' -replace '^[^/:]+(:\d+)?[/:]', ''
    $path = $path -replace '\.git/?$', '' -replace '/_git/', '/' -replace '^(scm|v\d+)/', ''
    return $path.Trim('/')
}

function Get-RepositoryRoot([string]$Folder) {
    $dir = $Folder
    while ($dir) {
        if (Test-Path -LiteralPath (Join-Path $dir '.git')) { return $dir }
        $parent = Split-Path $dir -Parent
        if ($parent -eq $dir) { break }
        $dir = $parent
    }
    return $null
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for VS Code to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    if ($fg -and $editors.Contains($fg.exe)) {
        try {
            if (([DateTime]::UtcNow - $script:workspacesAt).TotalMinutes -ge 5) {
                $dirs = @($editors.Values | ForEach-Object { Join-Path $env:APPDATA "$_\User\workspaceStorage" })
                $script:workspaces = Get-WorkspaceFolders $dirs
                $script:workspacesAt = [DateTime]::UtcNow
            }
            $ws = Get-WorkspaceFromTitle $fg.title @($script:workspaces.Keys)
            if ($ws.name) {
                $folder = $script:workspaces[$ws.name.ToLower()]
                $repo = $null
                if ($folder) {
                    $root = Get-RepositoryRoot $folder
                    if ($root -and (Test-Path -LiteralPath (Join-Path $root '.git\config'))) { $repo = Get-GitRemote (Get-Content -LiteralPath (Join-Path $root '.git\config') -Raw) }
                }
                $detail = @{ hwnd = $fg.hwnd; exe = $fg.exe }
                if ($folder) { $detail.path = $folder }
                if ($repo) { $detail.remote_folder = $repo }
                if ($ws.remote_server) { $detail.remote_server = $ws.remote_server }
                $key = "$($fg.hwnd)|$folder|$repo|$($ws.remote_server)"
                if ($detail.Count -gt 2 -and ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15)) {
                    Send-Message 'detail' $detail
                    $lastKey = $key
                    $lastSent = [DateTime]::UtcNow
                }
                Send-Status 'running' "Knows $($script:workspaces.Count) VS Code folders"
            }
        } catch {
            Send-Status 'degraded' "Could not read the workspace: $($_.Exception.Message)"
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}

# Web addresses: the domain of the page in front in Edge, Chrome, Brave
# and other Chromium browsers.
#
# The address bar is read through UI Automation. Only the domain
# (portal.contoso.com) and Microsoft tenant, subscription and workspace IDs
# are sent, never the full address. They are matched like a mail domain: add
# them to the customer page under domains and Microsoft IDs.

. "$PSScriptRoot/protocol.ps1"

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

$defaultBrowsers = @('msedge.exe', 'chrome.exe', 'brave.exe', 'vivaldi.exe', 'opera.exe', 'arc.exe')
$internalSchemes = '^(edge|chrome|brave|vivaldi|opera|about|file|data|javascript|view-source|devtools|chrome-extension|extension)[:]'
$script:addressBars = @{}

# The domain in an address-bar text, or $null for browser pages, local
# files and typed search words.
function Get-WebHost([string]$Address) {
    if (-not $Address) { return $null }
    $a = $Address.Trim()
    if ($a -match $internalSchemes) { return $null }
    $a = $a -replace '^[a-z][a-z0-9+.-]*://', ''
    $hostPart = ($a -split '[/?#]')[0] -replace '^[^@]*@', '' -replace ':\d+$', ''
    if ($hostPart -notmatch '^[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+$') { return $null }
    return ($hostPart.ToLower() -replace '^www\.', '')
}

$guid = '[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}'
$script:seenIds = @{}

# Microsoft IDs in an address, as objects with Kind and Id:
# - tenant: "#@contoso.onmicrosoft.com" or "#@<guid>" in Azure and Entra
#   portal addresses, "ctid=<guid>" in Power BI and Fabric addresses
#   (and tenantId= in other Microsoft portals)
# - subscription: "/subscriptions/<guid>" in Azure portal addresses
# - workspace: "/groups/<guid>" in Power BI and Fabric addresses
function Get-MicrosoftIds([string]$Address) {
    $patterns = @(
        @('tenant', "#@(?<id>$guid|[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+)"),
        @('tenant', "(?i)[?&#](ctid|tid|tenantId|tenant)=(?<id>$guid)"),
        @('subscription', "(?i)/subscriptions/(?<id>$guid)")
    )
    if ((Get-WebHost $Address) -match '(^|\.)(powerbi\.com|fabric\.microsoft\.com)$') {
        $patterns += , @('workspace', "(?i)/groups/(?<id>$guid)")
    }
    $out = @()
    $seen = @{}
    foreach ($p in $patterns) {
        foreach ($m in [regex]::Matches($Address, $p[1])) {
            $id = $m.Groups['id'].Value.ToLower()
            if ($seen.ContainsKey($id)) { continue }
            $seen[$id] = $true
            $out += [pscustomobject]@{ Kind = $p[0]; Id = $id }
        }
    }
    return $out
}

# The domains an address says something about: its host (SharePoint admin
# and OneDrive hosts as the customer's SharePoint host), plus the Microsoft
# tenant, subscription and workspace IDs in it (see Get-MicrosoftIds).
function Get-WebDomains([string]$Address) {
    $domainHost = Get-WebHost $Address
    if (-not $domainHost) { return @() }
    $out = @($domainHost -replace '^([a-z0-9]+)-(admin|my)\.sharepoint\.com$', '$1.sharepoint.com')
    $out += @(Get-MicrosoftIds $Address | ForEach-Object { $_.Id })
    return @($out | Select-Object -Unique)
}

# Logs each Microsoft ID the first time it is seen, so it can be copied to
# the right customer page.
function Write-NewIds([string]$Address) {
    $domainHost = Get-WebHost $Address
    foreach ($i in @(Get-MicrosoftIds $Address)) {
        if ($script:seenIds.ContainsKey($i.Id)) { continue }
        if ($script:seenIds.Count -gt 500) { $script:seenIds.Clear() }
        $script:seenIds[$i.Id] = $true
        Send-Log "Seen $($i.Kind) $($i.Id) on $domainHost"
    }
}

function Test-IgnoredDomain([string]$Domain, $Ignore) {
    foreach ($i in @($Ignore)) {
        $d = ([string]$i).Trim().ToLower().TrimStart('@').TrimStart('.')
        if ($d -and ($Domain -eq $d -or $Domain.EndsWith(".$d"))) { return $true }
    }
    return $false
}

# The browser's address bar: the first text field, which in Chromium comes
# before the page. Kept per window, as finding it takes a moment.
function Get-AddressBar([long]$Hwnd) {
    $bar = $script:addressBars[$Hwnd]
    if ($bar) {
        try { [void]$bar.Current.Name; return $bar } catch { $script:addressBars.Remove($Hwnd) }
    }
    $root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]$Hwnd)
    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Edit)
    $bar = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    if ($bar) {
        if ($script:addressBars.Count -gt 50) { $script:addressBars.Clear() }
        $script:addressBars[$Hwnd] = $bar
    }
    return $bar
}

function Get-AddressText($Bar) {
    $pattern = $Bar.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    return [string]$pattern.Current.Value
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for a browser to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    $browsers = @(Get-Setting 'browsers' $defaultBrowsers | ForEach-Object { $_.ToLower() })
    if ($fg -and $browsers -contains $fg.exe) {
        try {
            $bar = Get-AddressBar $fg.hwnd
            if ($null -eq $bar) {
                Send-Status 'degraded' 'The address bar of this browser could not be found'
            } else {
                $ignore = Get-Setting 'ignore_domains' @()
                $address = Get-AddressText $bar
                Write-NewIds $address
                $domains = @(Get-WebDomains $address | Where-Object { -not (Test-IgnoredDomain $_ $ignore) })
                if ($domains.Count) {
                    $key = "$($fg.hwnd)|$($domains -join ',')"
                    if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                        Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; mail_domains = $domains }
                        $lastKey = $key
                        $lastSent = [DateTime]::UtcNow
                    }
                }
                Send-Status 'running' 'Reading the address bar'
            }
        } catch {
            $script:addressBars.Remove([long]$fg.hwnd)
            Send-Status 'degraded' "Could not read the address bar: $($_.Exception.Message)"
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}

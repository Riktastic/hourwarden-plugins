# Web addresses: the domain of the page in front in Edge, Chrome, Brave
# and other Chromium browsers.
#
# The address bar is read through UI Automation. Only the domain is sent
# (portal.contoso.com), never the full address. It is matched like a mail
# domain: add the customer's web domains (contoso.com, contoso.sharepoint.com)
# on the customer page under mail domains.

. "$PSScriptRoot\protocol.ps1"

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
                $domain = Get-WebHost (Get-AddressText $bar)
                if ($domain -and -not (Test-IgnoredDomain $domain (Get-Setting 'ignore_domains' @()))) {
                    $key = "$($fg.hwnd)|$domain"
                    if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                        Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; mail_domains = @($domain) }
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

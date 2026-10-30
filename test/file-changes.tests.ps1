. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'file-changes')

$folder = Join-Path $env:TEMP "tk-file-changes-$PID"
New-Item -ItemType Directory -Force (Join-Path $folder 'sub') | Out-Null

Test 'normal documents are reported' {
    Assert-True (-not (Test-Ignored "$folder\Offerte.docx"))
    Assert-True (-not (Test-Ignored "$folder\sub\plan.xlsx"))
}

Test 'temporary files, hidden and tool folders and folders themselves are ignored' {
    foreach ($path in @("$folder\~`$Offerte.docx", "$folder\x.tmp", "$folder\download.crdownload", "$folder\.git\index", "$folder\app\node_modules\a.js", "$folder\.obsidian\workspace.json", "$folder\sub")) {
        Assert-True (Test-Ignored $path) "$path should be ignored"
    }
}

Test 'ignore patterns come from the settings' {
    $script:settings = [pscustomobject]@{ ignore_patterns = @('*.log') }
    Assert-True (Test-Ignored "$folder\build.log")
    Assert-True (-not (Test-Ignored "$folder\x.tmp")) 'the defaults are replaced'
    $script:settings = $null
}

Test 'a burst of changes goes quiet for ten seconds' {
    $start = [DateTime]::UtcNow
    $quiet = $false
    for ($i = 0; $i -lt 21; $i++) { $quiet = Test-Burst $start.AddMilliseconds($i * 10) }
    Assert-True $quiet 'the 21st change within 5 seconds is a burst'
    Assert-True (Test-Burst $start.AddSeconds(9)) 'still quiet'
    $script:burst.Clear()
    Assert-True (-not (Test-Burst $start.AddSeconds(11))) 'quiet period over'
}

Test 'a few saves are not a burst' {
    $script:burst.Clear()
    $script:quietUntil = [DateTime]::MinValue
    $start = [DateTime]::UtcNow
    foreach ($i in 1..5) { Assert-True (-not (Test-Burst $start.AddSeconds($i))) }
}

Remove-Item -Recurse -Force $folder
Complete-Tests

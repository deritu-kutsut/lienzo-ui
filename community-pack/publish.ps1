<#
.SYNOPSIS
  Publishes the Lienzo UI community pack (plugins and free fonts) to the Lienzo UI community.

.DESCRIPTION
  Uploads every plugin folder of ..\plugins (the new ones; add -IncludeBundled for the three older examples) and every font listed in
  fonts.json with the command line of the app:

    lienzo-ui.exe --community-upload <plugin|font> <path> --title "..." --description "..." --labels a,b

  It uses the session saved by the app (Community tab, sign in once). CLOSE THE APP FIRST: the session token rotates.
  Every upload is moderated by the server (it can end approved, pending or rejected: the message printed by the app says which).
  The script waits between uploads so the server rate limits are not hit, and writes publish-log.txt next to itself.

.PARAMETER WhatIf
  Only prints the commands, uploads nothing and does not need a saved session.

.PARAMETER Only
  all (default), plugins or fonts.

.PARAMETER Filter
  Wildcard on the plugin folder name or the font family, for example -Filter "press*" or -Filter "*grid*".

.PARAMETER DelaySeconds
  Pause between uploads (default 20).

.PARAMETER Exe
  Path of lienzo-ui.exe (default ..\target\release\lienzo-ui.exe).

.EXAMPLE
  powershell -NoProfile -File community-pack\publish.ps1 -WhatIf
  powershell -NoProfile -File community-pack\publish.ps1 -Only fonts
  powershell -NoProfile -File community-pack\publish.ps1 -Only plugins -Filter "mirror*"
#>
[CmdletBinding()]
param(
    [switch]$WhatIf,
    [ValidateSet("all", "plugins", "fonts")][string]$Only = "all",
    [string]$Filter = "*",
    [int]$DelaySeconds = 20,
    [string]$Exe = "",
    [switch]$IncludeBundled
)

$ErrorActionPreference = "Stop"
$pack = $PSScriptRoot
$repo = Split-Path -Parent $pack
if (-not $Exe) { $Exe = Join-Path $repo "target\release\lienzo-ui.exe" }
$log = Join-Path $pack "publish-log.txt"

# Labels of each plugin (at most 8, lowercase, 30 characters each).
$pluginLabels = @{
    "rename-by-hierarchy"   = "naming,hierarchy,cleanup,unity"
    "recolor-palette"       = "color,palette,theme,design"
    "placeholder-text"      = "text,lorem-ipsum,prototype,mockup"
    "grid-generator"        = "grid,inventory,slots,layout"
    "mirror-layout"         = "rtl,mirror,localization,layout"
    "distribute-siblings"   = "spacing,align,distribute,layout"
    "accessibility-report"  = "accessibility,contrast,wcag,exporter"
    "design-spec-markdown"  = "documentation,markdown,spec,exporter"
    "localization-sheet"    = "localization,translation,tsv,exporter"
    "safe-area-guides"      = "mobile,safe-area,notch,guides"
    "button-set-generator"  = "buttons,menu,generator,layout"
    "size-normalizer"       = "size,align,buttons,cleanup"
    "text-case-tools"       = "text,uppercase,title-case,tools"
    "palette-export"        = "color,palette,uss,exporter"
    "snap-to-grid"          = "grid,snap,align"
    "text-to-csv"           = "text,csv,translation,exporter"
    "extra-templates"       = "templates,shop,rpg"
}
$bundled = @("snap-to-grid", "text-to-csv", "extra-templates")

$jobs = New-Object System.Collections.Generic.List[object]

if ($Only -ne "fonts") {
    foreach ($dir in (Get-ChildItem -Directory (Join-Path $repo "plugins") | Sort-Object Name)) {
        if (-not (Test-Path (Join-Path $dir.FullName "plugin.json"))) { continue }
        if (($bundled -contains $dir.Name) -and -not $IncludeBundled) { continue }
        if ($dir.Name -notlike $Filter) { continue }
        $m = Get-Content -Raw -Encoding UTF8 (Join-Path $dir.FullName "plugin.json") | ConvertFrom-Json
        $parts = @()
        foreach ($c in @($m.commands)) { if ($c) { $parts += "command: " + $c.name } }
        foreach ($x in @($m.exporters)) { if ($x) { $parts += "exporter: " + $x.name + " (." + $x.extension + ")" } }
        $desc = ($m.description + " Adds " + ($parts -join "; ") + ". Made by Lienzo UI, MIT license.")
        if ($desc.Length -gt 900) { $desc = $desc.Substring(0, 897) + "..." }
        $labels = $pluginLabels[$dir.Name]
        if (-not $labels) { $labels = "plugin" }
        $jobs.Add([pscustomobject]@{ Kind = "plugin"; Name = $dir.Name; Path = $dir.FullName; Title = [string]$m.name; Description = $desc; Labels = $labels; License = "mit"; Source = $null })
    }
}

if ($Only -ne "plugins") {
    $fonts = Get-Content -Raw -Encoding UTF8 (Join-Path $pack "fonts.json") | ConvertFrom-Json
    foreach ($f in $fonts.fonts) {
        if ($f.family -notlike $Filter -and $f.slug -notlike $Filter) { continue }
        $copyright = if ($f.copyright) { $f.copyright.TrimEnd(",", ".", " ") } else { "Designer: " + $f.authors }
        $desc = ("{0} by {1}. License: {2} - free for personal and commercial use, including games and apps. Source: {3} - {4}. The license text is bundled with the font." -f $f.family, $f.authors, $f.license, $f.source, $copyright)
        if ($desc.Length -gt 900) { $desc = $desc.Substring(0, 897) + "..." }
        $labels = (@("font") + @($f.labels) + @($f.license_spdx.ToLower())) -join ","
        $jobs.Add([pscustomobject]@{ Kind = "font"; Name = $f.family; Path = (Join-Path $pack ($f.file -replace "/", "\")); Title = [string]$f.family; Description = $desc; Labels = $labels; License = "ofl"; Source = [string]$f.source })
    }
}

if ($jobs.Count -eq 0) { Write-Host "Nothing to upload (check -Only and -Filter)."; exit 0 }

if (-not $WhatIf) {
    if (-not (Test-Path $Exe)) { Write-Error "Not found: $Exe . Build it with: cargo build --release"; exit 1 }
    if (Get-Process -Name "lienzo-ui" -ErrorAction SilentlyContinue) { Write-Error "Close Lienzo UI first: the session token rotates and the open app would lose its session."; exit 1 }
}

function Quote-Arg([string]$s) { '"' + ($s -replace '"', '\"') + '"' }

$ok = 0; $failed = 0; $i = 0
"== $(Get-Date -Format s) publish $($jobs.Count) items" | Add-Content -Encoding UTF8 $log
foreach ($j in $jobs) {
    $i++
    $argList = @("--community-upload", $j.Kind, (Quote-Arg $j.Path), "--title", (Quote-Arg $j.Title), "--description", (Quote-Arg $j.Description), "--labels", (Quote-Arg $j.Labels), "--license", $j.License, "--rights")
    if ($j.Source) { $argList += @("--source", (Quote-Arg $j.Source)) }
    $line = "[{0}/{1}] {2}: {3}" -f $i, $jobs.Count, $j.Kind, $j.Name
    if ($WhatIf) {
        Write-Host $line
        Write-Host ("  & `"{0}`" {1}" -f $Exe, ($argList -join " "))
        continue
    }
    Write-Host $line -NoNewline
    $outFile = [System.IO.Path]::GetTempFileName(); $errFile = [System.IO.Path]::GetTempFileName()
    try {
        $p = Start-Process -FilePath $Exe -ArgumentList $argList -Wait -PassThru -NoNewWindow -RedirectStandardOutput $outFile -RedirectStandardError $errFile
        $text = ((Get-Content -Raw $outFile) + (Get-Content -Raw $errFile)).Trim()
        if ($p.ExitCode -eq 0) { $ok++; Write-Host "  OK  $text" -ForegroundColor Green } else { $failed++; Write-Host "  FAILED ($($p.ExitCode))  $text" -ForegroundColor Red }
        "$line | exit $($p.ExitCode) | $text" | Add-Content -Encoding UTF8 $log
    } finally {
        Remove-Item -Force -ErrorAction SilentlyContinue $outFile, $errFile
    }
    if ($i -lt $jobs.Count) { Start-Sleep -Seconds $DelaySeconds }
}

if ($WhatIf) {
    Write-Host ""
    Write-Host "WhatIf: $($jobs.Count) uploads would be made. Nothing was sent."
} else {
    Write-Host ""
    Write-Host "Done: $ok uploaded, $failed failed. Log: $log"
    Write-Host "Items left pending can be re-moderated with: `"$Exe`" --community-retry"
    if ($failed -gt 0) { exit 1 }
}

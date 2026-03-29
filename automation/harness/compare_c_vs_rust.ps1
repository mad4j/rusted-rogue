param(
    [string[]]$Scenarios = @("gs01_new_game", "gs02_move_hjkl", "gs03_inventory"),
    [int]$Runs = 3,
    [int]$TimeoutSec = 5,
    [int]$Seed = 12345
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$outDir = Join-Path $PSScriptRoot "out"
$inputDir = Join-Path $PSScriptRoot "inputs"
$reportPath = Join-Path $outDir "compare-report.md"

New-Item -ItemType Directory -Path $outDir -Force | Out-Null

$rows = @()

foreach ($scenario in $Scenarios) {
    $inputFile = Join-Path $inputDir "$scenario.txt"
    if (-not (Test-Path $inputFile)) {
        $rows += [pscustomobject]@{
            Scenario = $scenario
            CDeterministic = "FAIL"
            RustDeterministic = "FAIL"
            Comparative = "FAIL"
            Notes = "missing input file"
        }
        continue
    }

    $cPass = $false
    $cNotes = ""
    try {
        & (Join-Path $PSScriptRoot "run_golden.ps1") -Scenario $scenario -Runs $Runs -TimeoutSec $TimeoutSec -Seed $Seed
        $cPass = ($LASTEXITCODE -eq 0)
        if (-not $cPass) {
            $cNotes = "run_golden exit=$LASTEXITCODE"
        }
    }
    catch {
        $cPass = $false
        $cNotes = $_.Exception.Message
    }

    $rustPass = $true
    $rustSummaries = @()

    for ($i = 1; $i -le $Runs; $i++) {
        $env:RUSTED_ROGUE_SCRIPT_FILE = (Resolve-Path $inputFile).Path
        $env:RUSTED_ROGUE_SEED = "$Seed"

        $output = ""
        $rustExit = 0
        try {
            $output = (& cargo run --quiet 2>&1 | Out-String)
            $rustExit = $LASTEXITCODE
        }
        catch {
            $rustPass = $false
            $rustExit = 1
            $output = $_.Exception.Message
        }
        finally {
            Remove-Item Env:RUSTED_ROGUE_SCRIPT_FILE -ErrorAction SilentlyContinue
            Remove-Item Env:RUSTED_ROGUE_SEED -ErrorAction SilentlyContinue
        }

        if ($rustExit -ne 0) {
            $rustPass = $false
            $rustSummaries += "run${i}:exit=$rustExit"
            continue
        }

        $summary = ($output -split "`r?`n" | Where-Object { $_ -like "scenario_summary*" } | Select-Object -Last 1)
        if ([string]::IsNullOrWhiteSpace($summary)) {
            $rustPass = $false
            $rustSummaries += "run${i}:missing-summary"
            continue
        }

        $rustSummaries += $summary.Trim()
    }

    if ($rustPass -and $rustSummaries.Count -gt 1) {
        $baseline = $rustSummaries[0]
        foreach ($summary in $rustSummaries) {
            if ($summary -ne $baseline) {
                $rustPass = $false
                break
            }
        }
    }

    $comparativePass = $cPass -and $rustPass

    $notes = @()
    if (-not $cPass -and $cNotes) {
        $notes += "c:$cNotes"
    }
    if (-not $rustPass) {
        $notes += "rust:" + (($rustSummaries | Select-Object -First 2) -join " | ")
    }
    if ($notes.Count -eq 0) {
        $notes += "ok"
    }

    $rows += [pscustomobject]@{
        Scenario = $scenario
        CDeterministic = $(if ($cPass) { "PASS" } else { "FAIL" })
        RustDeterministic = $(if ($rustPass) { "PASS" } else { "FAIL" })
        Comparative = $(if ($comparativePass) { "PASS" } else { "FAIL" })
        Notes = ($notes -join "; ")
    }
}

$lines = @(
    "# C vs Rust Golden Scenario Report",
    "",
    "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')",
    "",
    "| Scenario | C deterministic | Rust deterministic | Comparative | Notes |",
    "|---|---|---|---|---|"
)

foreach ($row in $rows) {
    $lines += "| $($row.Scenario) | $($row.CDeterministic) | $($row.RustDeterministic) | $($row.Comparative) | $($row.Notes) |"
}

Set-Content -Path $reportPath -Value $lines -Encoding UTF8
Write-Output "Report written to $reportPath"

if ($rows.Where({ $_.Comparative -eq "FAIL" }).Count -gt 0) {
    exit 3
}

exit 0

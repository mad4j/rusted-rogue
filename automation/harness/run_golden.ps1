param(
    [string]$Scenario = "gs01_new_game",
    [int]$Runs = 3,
    [int]$TimeoutSec = 5,
    [int]$Seed = 12345
)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$wslScript = "$root/automation/harness/run_golden_wsl.sh"

# Validate WSL availability first
$wslCmd = Get-Command wsl -ErrorAction SilentlyContinue
if (-not $wslCmd) {
    Write-Error "WSL command not found. Install/enable WSL first."
    exit 1
}

# Convert Windows path to WSL path without relying on wslpath parsing of backslashes.
# Example: C:\Users\name\repo\file.sh -> /mnt/c/Users/name/repo/file.sh
$resolvedScript = (Resolve-Path $wslScript).Path
$driveLetter = $resolvedScript.Substring(0, 1).ToLower()
$rest = $resolvedScript.Substring(2).Replace('\', '/')
$wslScriptPath = "/mnt/$driveLetter$rest"

wsl bash "$wslScriptPath" "$Scenario" "$Runs" "$TimeoutSec" "$Seed"
exit $LASTEXITCODE

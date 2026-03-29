# Golden Scenario Harness (BL-004)

This harness validates repeatability for baseline C scenarios by replaying scripted input, capturing terminal output, normalizing logs, and comparing hashes across 3 runs.

## Files
- `run_golden.ps1`: Windows entrypoint (calls WSL).
- `run_golden_wsl.sh`: Linux/WSL runner with `script` and `timeout`.
- `tools/normalize_log.py`: strips ANSI/control sequences and normalizes line endings.
- `inputs/*.txt`: scenario key streams.
- `out/`: generated raw/normalized logs and hashes.

## Prerequisites
- WSL enabled and configured.
- In WSL distro: `bash`, `script` (package `bsdutils` or `util-linux`), `timeout` (coreutils), `python3`, `sha256sum`.
- Legacy binary present at `original/rogue-libc5-ncurses/rogue/rogue`.

## Run
From repository root in PowerShell:

```powershell
./automation/harness/run_golden.ps1 -Scenario gs01_new_game -Runs 3 -TimeoutSec 5
```

Comparative C vs Rust report (scenario-level pass/fail):

```powershell
./automation/harness/compare_c_vs_rust.ps1 -Runs 3 -TimeoutSec 5 -Seed 12345
```

Other scenarios currently seeded:
- `gs02_move_hjkl`
- `gs03_inventory`

## Result
- PASS: all normalized output hashes match.
- FAIL: at least one run differs.

For comparative report:
- Output: `automation/harness/out/compare-report.md`
- Exit code `0`: all scenarios comparative PASS
- Exit code `3`: at least one scenario comparative FAIL

## Notes
- This is a minimum harness to unblock BL-004.
- You can expand inputs to cover full GS-01..GS-07 set.

## Troubleshooting
- Error: `./rogue: No such file or directory` in WSL
Cause: legacy prebuilt binary targets old libc5 interpreter (`/lib/ld-linux.so.1`).
Fix: rebuild from source inside WSL.

- Error: `fatal error: curses.h: No such file or directory` during `make`
Cause: ncurses development headers missing.
Fix (Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install build-essential libncurses5-dev
cd /mnt/c/Users/danie/Documents/GitHub/rusted-rogue/original/rogue-libc5-ncurses/rogue
make
```

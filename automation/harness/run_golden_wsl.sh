#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_DIR="$ROOT_DIR/automation/harness"
OUT_DIR="$HARNESS_DIR/out"
INPUT_DIR="$HARNESS_DIR/inputs"
TOOLS_DIR="$HARNESS_DIR/tools"
GAME_DIR="$ROOT_DIR/original/rogue-libc5-ncurses/rogue"
BIN="$GAME_DIR/rogue"

SCENARIO="${1:-gs01_new_game}"
INPUT_FILE="$INPUT_DIR/${SCENARIO}.txt"
RUNS="${2:-3}"
TIMEOUT_SEC="${3:-5}"
SEED="${4:-12345}"

if [[ ! -x "$BIN" ]]; then
  echo "ERROR: binary not executable: $BIN"
  exit 1
fi

if [[ ! -f "$INPUT_FILE" ]]; then
  echo "ERROR: scenario input not found: $INPUT_FILE"
  exit 1
fi

mkdir -p "$OUT_DIR"

for i in $(seq 1 "$RUNS"); do
  raw="$OUT_DIR/${SCENARIO}.run${i}.raw.log"
  norm="$OUT_DIR/${SCENARIO}.run${i}.norm.log"

  # Feed keys to rogue and capture terminal output.
  # We use script for tty capture and timeout to avoid hangs.
  status=0
  (
    cd "$GAME_DIR"
    timeout "$TIMEOUT_SEC" bash -lc "cat '$INPUT_FILE' | ROGUE_SEED='$SEED' script -qec './rogue' '$raw'"
  ) || status=$?

  # timeout(124) is acceptable for this harness because rogue is interactive.
  if [[ "$status" -ne 0 && "$status" -ne 124 ]]; then
    echo "FAIL: scenario $SCENARIO run${i} exited with status $status"
    exit 4
  fi

  if [[ ! -s "$raw" ]]; then
    echo "FAIL: scenario $SCENARIO run${i} produced empty raw log"
    exit 5
  fi

  if grep -qiE "no such file or directory|not found|exec format error" "$raw"; then
    echo "FAIL: scenario $SCENARIO run${i} could not execute rogue binary"
    exit 6
  fi

  python3 "$TOOLS_DIR/normalize_log.py" "$raw" "$norm"
  sha256sum "$norm" > "$OUT_DIR/${SCENARIO}.run${i}.sha256"
done

# Compare all hashes with run1
base="$(cut -d' ' -f1 "$OUT_DIR/${SCENARIO}.run1.sha256")"
for i in $(seq 2 "$RUNS"); do
  cur="$(cut -d' ' -f1 "$OUT_DIR/${SCENARIO}.run${i}.sha256")"
  if [[ "$cur" != "$base" ]]; then
    echo "FAIL: non-deterministic output for $SCENARIO (run1 != run${i})"
    exit 3
  fi
done

echo "PASS: deterministic output for $SCENARIO across $RUNS runs"

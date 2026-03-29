#!/usr/bin/env python3
import re
import sys
from pathlib import Path

if len(sys.argv) != 3:
    print("usage: normalize_log.py <in> <out>", file=sys.stderr)
    sys.exit(2)

src = Path(sys.argv[1])
dst = Path(sys.argv[2])
text = src.read_bytes().decode("utf-8", errors="ignore")

# Strip ANSI control sequences (CSI, OSC and single-char escapes)
text = re.sub(r"\x1B\[[0-?]*[ -/]*[@-~]", "", text)
text = re.sub(r"\x1B\][^\x07]*(\x07|\x1B\\)", "", text)
text = re.sub(r"\x1B[@-_]", "", text)

# Normalize line endings and trim trailing spaces
lines = [ln.rstrip() for ln in text.replace("\r\n", "\n").replace("\r", "\n").split("\n")]

# Drop script(1) session timestamp metadata that changes every run.
lines = [ln for ln in lines if not (ln.startswith("Script started on ") or ln.startswith("Script done on "))]

dst.write_text("\n".join(lines).strip() + "\n", encoding="utf-8")

#!/usr/bin/env python3
"""Parse an RTIC trace into a CSV tracking the resource ceiling evolution."""

import csv
import re
import sys

LINE_RE = re.compile(
    r"\[obs\]\s*@\s*(\d+)\s+"
    r"(act|acq|rel|comp)\s+"
    r"(\S+)"
    r"(?:\s+t=(\d+))?"
    r"(?:\s+c=(\d+))?"
)


def parse(trace_path, csv_path):
    ceiling_stack = []

    with open(trace_path) as f, open(csv_path, "w", newline="") as out:
        w = csv.writer(out)
        w.writerow(["ts", "event", "task", "priority", "ceiling", "ceiling_stack"])
        for line in f:
            m = LINE_RE.match(line.strip())
            if not m:
                continue
            ts, event, name, prio, ceil = m.groups()
            if event == "acq":
                ceiling_stack.append(int(ceil))
            elif event == "rel":
                if ceiling_stack:
                    ceiling_stack.pop()
            w.writerow([
                ts, event, name, prio or "", ceil or "",
                "-".join(str(c) for c in ceiling_stack) if ceiling_stack else "0",
            ])


if __name__ == "__main__":
    trace = sys.argv[1] if len(sys.argv) > 1 else "mutex.trace"
    out = sys.argv[2] if len(sys.argv) > 2 else "ceiling.csv"
    parse(trace, out)
    print(f"Written {out}")

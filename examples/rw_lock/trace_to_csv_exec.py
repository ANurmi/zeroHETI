#!/usr/bin/env python3
"""Parse an RTIC trace into a CSV of task execution intervals."""

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
    pending = {}  # task_name -> start_cycle

    with open(trace_path) as f, open(csv_path, "w", newline="") as out:
        w = csv.writer(out)
        w.writerow(["start", "end", "task", "priority"])
        for line in f:
            m = LINE_RE.match(line.strip())
            if not m:
                continue
            ts, event, name, prio, _ = m.groups()
            if event == "act":
                pending[name] = (int(ts), int(prio))
            elif event == "comp" and name in pending:
                start, priority = pending.pop(name)
                w.writerow([start, int(ts), name, priority])


if __name__ == "__main__":
    trace = sys.argv[1] if len(sys.argv) > 1 else "mutex.trace"
    out = sys.argv[2] if len(sys.argv) > 2 else "intervals.csv"
    parse(trace, out)
    print(f"Written {out}")

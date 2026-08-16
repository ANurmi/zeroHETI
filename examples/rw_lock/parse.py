#!/usr/bin/env python3
"""Extract per-job records from a rw_lock simulation stdout into a CSV.

Usage: parse.py <in.stdout> <out.csv>

Recognises the lines emitted by the firmware teardown:
  MODE <mode>
  JOB <task> <idx> <resp_us> <rel_us> <miss>
"""
import csv
import re
import sys

JOB_RE = re.compile(r"^JOB (\w+) (\d+) (\d+) (\d+) (\d+)")
MODE_RE = re.compile(r"^MODE (\w+)")


def main() -> None:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(1)

    infile, outfile = sys.argv[1], sys.argv[2]
    mode = "?"
    rows = []
    with open(infile, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            m = MODE_RE.match(line)
            if m:
                mode = m.group(1)
                continue
            m = JOB_RE.match(line)
            if m:
                task, idx, resp_us, rel_us, miss = m.groups()
                rows.append((mode, task, int(idx), int(resp_us), int(rel_us), int(miss)))

    if not rows:
        print(f"warning: no JOB lines found in {infile}", file=sys.stderr)
        sys.exit(1)

    with open(outfile, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["mode", "task", "idx", "resp_us", "rel_us", "miss"])
        w.writerows(rows)
    print(f"{infile}: {len(rows)} jobs ({mode}) -> {outfile}")


if __name__ == "__main__":
    main()

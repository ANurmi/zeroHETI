#!/usr/bin/env -S uv run
"""Check that the address map regions in mmap/global.yml's 'memory' section
(including regions pulled in from other mmap/ sources via 'include:')
do not conflict.

Checks performed:
  * every region fits in the address space (address_bits)
  * top-level regions do not overlap each other
  * children lie entirely inside their parent region
  * sibling children do not overlap each other

Children without an explicit 'start' are placed directly after the previous
sibling (or at the parent's start if they are the first child).

Usage: check_memory_map.py [path/to/mmap/global.yml]
Exits 0 if no conflicts are found, 1 otherwise.
"""
import re
import sys
from pathlib import Path

import yaml

from mmap_sources import DEFAULT_PATH, expand_regions, load_yaml

UNITS = {"": 1, "K": 1 << 10, "M": 1 << 20, "G": 1 << 30, "T": 1 << 40}
SIZE_RE = re.compile(r"^(\d+)\s*([KMGT]?)$", re.IGNORECASE)


def parse_size(value):
    if isinstance(value, int):
        return value
    m = SIZE_RE.match(str(value).strip())
    if not m:
        raise ValueError(f"invalid size '{value}'")
    return int(m.group(1)) * UNITS[m.group(2).upper()]


class Region:
    def __init__(self, rid, name, start, size, path):
        self.id = rid
        self.name = name
        self.start = start
        self.size = size
        self.end = start + size  # exclusive
        self.path = path
        self.children = []

    def __str__(self):
        return f"{self.path} [0x{self.start:08X} - 0x{self.end - 1:08X}]"


def build(raw, path, default_start=None):
    rid = raw.get("id", "?")
    path = f"{path}.{rid}" if path else rid
    start = raw.get("start", default_start)
    if start is None:
        raise ValueError(f"{path}: no 'start' and none can be inferred")
    region = Region(rid, raw.get("name", rid), start, parse_size(raw["size"]), path)
    next_start = region.start
    for child in raw.get("children", []) or []:
        c = build(child, path, default_start=next_start)
        region.children.append(c)
        next_start = c.end
    return region


def check_overlaps(regions, scope, errors):
    ordered = sorted(regions, key=lambda r: r.start)
    for a, b in zip(ordered, ordered[1:]):
        if b.start < a.end:
            errors.append(f"overlap in {scope}: {a} and {b}")


def check_tree(region, errors):
    for c in region.children:
        if c.start < region.start or c.end > region.end:
            errors.append(f"{c} lies outside parent {region}")
    check_overlaps(region.children, region.path, errors)
    for c in region.children:
        check_tree(c, errors)


def print_map(regions, indent=0):
    for r in sorted(regions, key=lambda r: r.start):
        print(f"  {'  ' * indent}0x{r.start:08X} - 0x{r.end - 1:08X}  {r.name} ({r.id})")
        print_map(r.children, indent + 1)


def main():
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PATH
    try:
        doc = load_yaml(path)
        mem = doc["memory"]
        addr_bits = mem["address_bits"]
        regions = [build(r, "") for r in expand_regions(mem["regions"], path)]
    except (OSError, yaml.YAMLError, KeyError, TypeError, ValueError) as e:
        print(f"ERROR: cannot load memory map from {path}: {e!r}")
        print("Hint: run check_syntax.py first.")
        return 1

    errors = []
    limit = 1 << addr_bits

    def check_bounds(r):
        if r.size <= 0:
            errors.append(f"{r} has non-positive size")
        if r.start < 0 or r.end > limit:
            errors.append(f"{r} exceeds {addr_bits}-bit address space")
        for c in r.children:
            check_bounds(c)

    for r in regions:
        check_bounds(r)
    check_overlaps(regions, "memory.regions", errors)
    for r in regions:
        check_tree(r, errors)

    print(f"{path}: memory map ({addr_bits}-bit)")
    print_map(regions)
    if errors:
        print(f"\n{len(errors)} conflict(s):")
        for e in errors:
            print(f"  ERROR: {e}")
        return 1
    print("\nNo address map conflicts.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

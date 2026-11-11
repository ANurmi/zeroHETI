#!/usr/bin/env -S uv run
"""Check the general syntax/structure of the memory map sources in mmap/.

Starts from mmap/global.yml and follows 'include:' entries into the other
sources (e.g. mmap/peripherals.yml), checking each one. Region ids must be
unique across all sources.

Usage: check_syntax.py [path/to/mmap/global.yml]
Exits 0 if the file is valid, 1 otherwise.
"""
import re
import sys
from pathlib import Path

import yaml

from mmap_sources import DEFAULT_PATH, SOURCE_KEYS, include_path, is_include, load_yaml

SIZE_RE = re.compile(r"^(\d+)\s*([KMGT]?)$", re.IGNORECASE)

errors = []


def err(where, msg):
    errors.append(f"{where}: {msg}")


def check_type(where, value, types, name):
    if not isinstance(value, types) or isinstance(value, bool):
        err(where, f"'{name}' has wrong type {type(value).__name__}")
        return False
    return True


def check_size(where, value):
    if isinstance(value, int) and not isinstance(value, bool):
        if value <= 0:
            err(where, "'size' must be positive")
    elif isinstance(value, str):
        if not SIZE_RE.match(value.strip()):
            err(where, f"'size' '{value}' is not a number or <N>[K|M|G|T]")
    else:
        err(where, f"'size' has wrong type {type(value).__name__}")


def check_include(where, entry, seen_ids, start_required, src, stack):
    """Check an '- include: <file>' entry and the source it refers to."""
    for key in entry:
        if key != "include":
            err(where, f"unknown key '{key}' in include entry")
    if not check_type(where, entry["include"], str, "include"):
        return
    target = include_path(entry, src)
    if target in stack:
        chain = " -> ".join(p.name for p in stack + (target,))
        err(where, f"include cycle: {chain}")
        return
    try:
        doc = load_yaml(target)
    except OSError as e:
        err(where, f"cannot read included source {target}: {e}")
        return
    except yaml.YAMLError as e:
        err(where, f"YAML parse error in included source {target}:\n{e}")
        return

    name = target.name
    if not isinstance(doc, dict):
        err(name, "top level must be a mapping")
        return
    for key in doc:
        if key not in SOURCE_KEYS:
            err(name, f"unknown key '{key}'")
    check_version(name, doc)
    regions = doc.get("regions")
    if not isinstance(regions, list) or not regions:
        err(name, "'regions' must be a non-empty list")
        return
    for i, region in enumerate(regions):
        check_region(f"{name}:regions[{i}]", region, seen_ids, start_required,
                     target, stack + (target,))


def check_version(where, doc):
    if "version" not in doc:
        err(where, "missing 'version'")
    elif check_type(where, doc["version"], int, "version") and doc["version"] != 1:
        err(where, f"unsupported version {doc['version']}")


def check_region(where, region, seen_ids, start_required, src, stack):
    if is_include(region):
        check_include(where, region, seen_ids, start_required, src, stack)
        return
    if not isinstance(region, dict):
        err(where, "region must be a mapping")
        return
    allowed = {"id", "name", "start", "size", "children"}
    for key in region:
        if key not in allowed:
            err(where, f"unknown key '{key}'")

    for key in ("id", "name", "size"):
        if key not in region:
            err(where, f"missing required key '{key}'")

    rid = region.get("id")
    if rid is not None and check_type(where, rid, str, "id"):
        where = f"{where} ({rid})"
        if rid in seen_ids:
            err(where, f"duplicate id '{rid}'")
        seen_ids.add(rid)

    if "name" in region:
        check_type(where, region["name"], str, "name")

    if "start" in region:
        if check_type(where, region["start"], int, "start") and region["start"] < 0:
            err(where, "'start' must be non-negative")
    elif start_required:
        err(where, "missing required key 'start'")

    if "size" in region:
        check_size(where, region["size"])

    if "children" in region:
        children = region["children"]
        if not isinstance(children, list) or not children:
            err(where, "'children' must be a non-empty list")
        else:
            for i, child in enumerate(children):
                check_region(f"{where}.children[{i}]", child, seen_ids,
                             start_required=False, src=src, stack=stack)


def main():
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PATH
    try:
        doc = load_yaml(path)
    except OSError as e:
        print(f"ERROR: cannot read {path}: {e}")
        return 1
    except yaml.YAMLError as e:
        print(f"ERROR: YAML parse error in {path}:\n{e}")
        return 1

    if not isinstance(doc, dict):
        print(f"ERROR: {path}: top level must be a mapping")
        return 1

    for key in doc:
        if key not in ("version", "figure", "memory"):
            err("top level", f"unknown key '{key}'")

    # version
    check_version("top level", doc)

    # figure (optional)
    fig = doc.get("figure")
    if fig is not None:
        if not isinstance(fig, dict):
            err("figure", "must be a mapping")
        else:
            for key, val in fig.items():
                if key in ("title", "footer"):
                    check_type("figure", val, str, key)
                elif key == "legend":
                    if not isinstance(val, bool):
                        err("figure", "'legend' must be true/false")
                else:
                    err("figure", f"unknown key '{key}'")

    # memory
    mem = doc.get("memory")
    if mem is None:
        err("top level", "missing 'memory'")
    elif not isinstance(mem, dict):
        err("memory", "must be a mapping")
    else:
        for key in mem:
            if key not in ("address_bits", "regions"):
                err("memory", f"unknown key '{key}'")
        bits = mem.get("address_bits")
        if bits is None:
            err("memory", "missing 'address_bits'")
        elif check_type("memory", bits, int, "address_bits") and not 1 <= bits <= 64:
            err("memory", f"'address_bits' {bits} out of range 1..64")

        regions = mem.get("regions")
        if not isinstance(regions, list) or not regions:
            err("memory", "'regions' must be a non-empty list")
        else:
            seen_ids = set()
            root = path.resolve()
            for i, region in enumerate(regions):
                check_region(f"memory.regions[{i}]", region, seen_ids,
                             start_required=True, src=root, stack=(root,))

    if errors:
        print(f"{path}: {len(errors)} error(s)")
        for e in errors:
            print(f"  ERROR: {e}")
        return 1
    print(f"{path}: syntax OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())

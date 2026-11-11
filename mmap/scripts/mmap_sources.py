"""Shared helpers for loading the memory map sources in mmap/.

The memory map is split over several YAML sources. The root source
(mmap/global.yml) holds 'version', 'figure' and 'memory'. Anywhere a region
may appear (memory.regions or a region's 'children'), an entry of the form

    - include: peripherals.yml

is replaced by the 'regions' list of the referenced source. Include paths are
relative to the including file. An included source looks like:

    version: 1
    regions:
      - { id: ..., name: ..., start: ..., size: ... }

Included sources may themselves contain include entries.
"""
from pathlib import Path

import yaml

MMAP_DIR = Path(__file__).resolve().parent.parent / "mmap"
DEFAULT_PATH = MMAP_DIR / "global.yml"
SOURCE_KEYS = ("version", "regions")


def is_include(entry):
    return isinstance(entry, dict) and "include" in entry


def include_path(entry, base_file):
    """Resolve an include entry's target relative to the including file."""
    return (Path(base_file).parent / entry["include"]).resolve()


def load_yaml(path):
    with open(path) as f:
        return yaml.safe_load(f)


def expand_regions(regions, base_file, stack=()):
    """Return 'regions' with all include entries (recursively) expanded.

    Raises ValueError on a malformed source or an include cycle.
    """
    base_file = Path(base_file).resolve()
    stack = stack + (base_file,)
    out = []
    for entry in regions or []:
        if is_include(entry):
            target = include_path(entry, base_file)
            if target in stack:
                chain = " -> ".join(p.name for p in stack + (target,))
                raise ValueError(f"include cycle: {chain}")
            src = load_yaml(target)
            if not isinstance(src, dict) or not isinstance(src.get("regions"), list):
                raise ValueError(f"{target}: source must be a mapping with a 'regions' list")
            out.extend(expand_regions(src["regions"], target, stack))
        elif isinstance(entry, dict) and "children" in entry:
            entry = dict(entry)
            entry["children"] = expand_regions(entry["children"], base_file, stack[:-1])
            out.append(entry)
        else:
            out.append(entry)
    return out

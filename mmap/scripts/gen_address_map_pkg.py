#!/usr/bin/env -S uv run
"""Generate the SystemVerilog package 'address_map_pkg.sv' from the memory map
sources in mmap/ (mmap/global.yml plus anything pulled in via 'include:').

Generated contents:
  * addr_rule_t            - {base, last} pair, AddrWidth bits each
  * <id>_map_t             - one packed struct per region that has children:
                             blanket 'base'/'last' fields covering the whole
                             region, a 'num' field, then one field per child
                             (nested regions get their own nested struct
                             type). This lets users write e.g.
                             AddressMap.periph.base without caring about the
                             children.
  * addr_map_t             - top-level struct: a 'num' field, then one field
                             per top-level region
  * <Id>Addr               - addr_rule_t localparam for every region,
                             including regions that have children
  * <Id>Map                - <id>_map_t localparam for every region with
                             children, built from its own <Id>Addr (blanket
                             rule) and its children's localparams
  * AddressMap             - addr_map_t localparam for the whole map

'num' (int unsigned) is the number of rules directly at that level, i.e. the
number of child fields; the blanket 'base'/'last' fields are not counted, and
a nested child struct counts as one regardless of its own contents.

Struct fields follow source order. 'last' is the exclusive end address,
i.e. the first address after the region (base + size). A region that ends at
the very top of the address space is rejected, since its 'last' would not fit
in AddrWidth bits.
Children without an explicit 'start' are placed exactly as in
check_memory_map.py. The map is checked for conflicts before generation.

With --prefix PREFIX the package is named '<PREFIX>_address_map_pkg' (and
the default output file '<PREFIX>_address_map_pkg.sv') instead.

Usage: gen_address_map_pkg.py [--prefix PREFIX] [-o OUTPUT] [path/to/mmap/global.yml]
Exits 0 on success, 1 on error.
"""
import argparse
import re
import sys
from pathlib import Path

import yaml

from check_memory_map import build, check_overlaps, check_tree
from mmap_sources import DEFAULT_PATH, expand_regions, load_yaml

PKG_NAME = "address_map_pkg"
REPO_ROOT = Path(__file__).resolve().parent.parent
IDENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
# Blanket rule fields placed at the start of every <id>_map_t; child ids must
# not reuse them.
BLANKET_FIELDS = ("base", "last")
# Rule count field present in every <id>_map_t and in addr_map_t.
NUM_FIELD = "num"
NUM_TYPE = "int unsigned"

# SystemVerilog (IEEE 1800-2017) reserved keywords; region ids are used
# verbatim as struct field names, so they must not collide with these.
SV_KEYWORDS = set("""
accept_on alias always always_comb always_ff always_latch and assert assign
assume automatic before begin bind bins binsof bit break buf bufif0 bufif1
byte case casex casez cell chandle checker class clocking cmos config const
constraint context continue cover covergroup coverpoint cross deassign
default defparam design disable dist do edge else end endcase endchecker
endclass endclocking endconfig endfunction endgenerate endgroup endinterface
endmodule endpackage endprimitive endprogram endproperty endspecify
endsequence endtable endtask enum event eventually expect export extends
extern final first_match for force foreach forever fork forkjoin function
generate genvar global highz0 highz1 if iff ifnone ignore_bins illegal_bins
implements implies import incdir include initial inout input inside instance
int integer interconnect interface intersect join join_any join_none large
let liblist library local localparam logic longint macromodule matches
medium modport module nand negedge nettype new nexttime nmos nor
noshowcancelled not notif0 notif1 null or output package packed parameter
pmos posedge primitive priority program property protected pull0 pull1
pulldown pullup pulsestyle_ondetect pulsestyle_onevent pure rand randc
randcase randsequence rcmos real realtime ref reg reject_on release repeat
restrict return rnmos rpmos rtran rtranif0 rtranif1 s_always s_eventually
s_nexttime s_until s_until_with scalared sequence shortint shortreal
showcancelled signed small soft solve specify specparam static string strong
strong0 strong1 struct super supply0 supply1 sync_accept_on sync_reject_on
table tagged task this throughout time timeprecision timeunit tran tranif0
tranif1 tri tri0 tri1 triand trior trireg type typedef union unique unique0
unsigned until until_with untyped use uwire var vectored virtual void wait
wait_order wand weak weak0 weak1 while wildcard wire with within wor xnor xor
""".split())


def camel(rid):
    """'uart0' -> 'Uart0', 'boot_rom' -> 'BootRom'."""
    return "".join(p[:1].upper() + p[1:] for p in rid.split("_") if p)


def rule_param(r):
    return f"{camel(r.id)}Addr"


def map_param(r):
    return f"{camel(r.id)}Map"


def map_type(r):
    return f"{r.id}_map_t"


def walk(regions):
    for r in regions:
        yield r
        yield from walk(r.children)


def check_names(regions):
    """Return a list of errors for ids/generated names that are not valid,
    unique SystemVerilog identifiers."""
    errors = []
    for r in regions:
        if r.id == NUM_FIELD:
            errors.append(f"{r.path}: id '{r.id}' clashes with the generated "
                          f"'{r.id}' field of addr_map_t")
    taken = {"addr_rule_t": "built-in type", "addr_map_t": "built-in type",
             "AddressMap": "built-in parameter", "AddrWidth": "built-in parameter"}
    for r in walk(regions):
        if not IDENT_RE.match(r.id):
            errors.append(f"{r.path}: id '{r.id}' is not a valid SystemVerilog identifier")
            continue
        if r.id in SV_KEYWORDS:
            errors.append(f"{r.path}: id '{r.id}' is a SystemVerilog keyword")
        for c in r.children:
            if c.id in BLANKET_FIELDS + (NUM_FIELD,):
                errors.append(f"{c.path}: id '{c.id}' clashes with the generated "
                              f"'{c.id}' field of {map_type(r)}")
        names = [rule_param(r)]
        if r.children:
            names += [map_param(r), map_type(r)]
        for n in names:
            if n in taken:
                errors.append(f"{r.path}: generated name '{n}' clashes with {taken[n]}")
            else:
                taken[n] = r.path
    return errors


def hex_lit(value, bits):
    digits = (bits + 3) // 4
    s = f"{value:0{digits}X}"
    # Group in 4-digit chunks from the right for readability.
    groups = []
    while s:
        groups.insert(0, s[-4:])
        s = s[:-4]
    return f"{bits}'h{'_'.join(groups)}"


def struct_typedef(name, fields):
    width = max(len(t) for t, _ in fields)
    lines = ["  typedef struct packed {"]
    lines += [f"    {t:<{width}} {f};" for t, f in fields]
    lines.append(f"  }} {name};")
    return lines


def assignment_pattern(ptype, pname, fields):
    width = max(len(f) for f, _ in fields) + 1
    lines = [f"  localparam {ptype} {pname} = '{{"]
    body = [f"    {f + ':':<{width}} {v}" for f, v in fields]
    lines.append(",\n".join(body))
    lines.append("  };")
    return lines


def field_type(r):
    return map_type(r) if r.children else "addr_rule_t"


def field_value(r):
    return map_param(r) if r.children else rule_param(r)


def struct_types(regions):
    """Typedefs for every region with children, innermost first so each type
    is declared before it is used."""
    out = []
    for r in regions:
        if r.children:
            out += struct_types(r.children)
            blanket = [("logic [AddrWidth-1:0]", f) for f in BLANKET_FIELDS]
            out += struct_typedef(map_type(r),
                                  blanket + [(NUM_TYPE, NUM_FIELD)]
                                  + [(field_type(c), c.id) for c in r.children])
            out.append("")
    return out


def map_params(regions):
    """Map localparams for every region with children, innermost first."""
    out = []
    for r in regions:
        if r.children:
            out += map_params(r.children)
            blanket = [(f, f"{rule_param(r)}.{f}") for f in BLANKET_FIELDS]
            out += assignment_pattern(map_type(r), map_param(r),
                                      blanket + [(NUM_FIELD, str(len(r.children)))]
                                      + [(c.id, field_value(c)) for c in r.children])
            out.append("")
    return out


def package_name(prefix):
    return f"{prefix}_{PKG_NAME}" if prefix else PKG_NAME


def generate(regions, addr_bits, src, pkg_name=PKG_NAME):
    all_regions = list(walk(regions))
    name_w = max(len(rule_param(r)) for r in all_regions)

    lines = [
        f"// Generated by scripts/gen_address_map_pkg.py from {src}.",
        "// DO NOT EDIT: edit the mmap/ sources and regenerate instead.",
        "",
        f"package {pkg_name};",
        "",
        f"  localparam int unsigned AddrWidth = {addr_bits};",
        "",
        "  typedef struct packed {",
        "    logic [AddrWidth-1:0] base;",
        "    logic [AddrWidth-1:0] last;",
        "  } addr_rule_t;",
        "",
    ]
    lines += struct_types(regions)
    lines += struct_typedef("addr_map_t", [(NUM_TYPE, NUM_FIELD)]
                            + [(field_type(r), r.id) for r in regions])
    lines.append("")

    for r in all_regions:
        lines.append(
            f"  localparam addr_rule_t {rule_param(r):<{name_w}} = "
            f"'{{base: {hex_lit(r.start, addr_bits)}, last: {hex_lit(r.end, addr_bits)}}};"
            f"  // {r.name}")
    lines.append("")

    lines += map_params(regions)
    lines += assignment_pattern("addr_map_t", "AddressMap",
                                [(NUM_FIELD, str(len(regions)))]
                                + [(r.id, field_value(r)) for r in regions])
    lines += ["", f"endpackage : {pkg_name}", ""]
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("source", nargs="?", type=Path, default=DEFAULT_PATH,
                    help=f"root memory map source (default: {DEFAULT_PATH})")
    ap.add_argument("-p", "--prefix",
                    help=f"package name prefix; generates '<PREFIX>_{PKG_NAME}' "
                         f"instead of '{PKG_NAME}'")
    ap.add_argument("-o", "--output", type=Path,
                    help="output file (default: <package name>.sv in the repo root)")
    args = ap.parse_args()

    if args.prefix is not None and not IDENT_RE.match(args.prefix):
        ap.error(f"--prefix '{args.prefix}' is not a valid SystemVerilog identifier")
    pkg_name = package_name(args.prefix)
    if args.output is None:
        args.output = REPO_ROOT / f"{pkg_name}.sv"

    try:
        doc = load_yaml(args.source)
        mem = doc["memory"]
        addr_bits = mem["address_bits"]
        regions = [build(r, "") for r in expand_regions(mem["regions"], args.source)]
    except (OSError, yaml.YAMLError, KeyError, TypeError, ValueError) as e:
        print(f"ERROR: cannot load memory map from {args.source}: {e!r}")
        print("Hint: run check_syntax.py first.")
        return 1

    errors = []
    limit = 1 << addr_bits
    for r in walk(regions):
        if r.size <= 0 or r.start < 0 or r.end > limit:
            errors.append(f"{r} is empty or exceeds the {addr_bits}-bit address space")
        elif r.end == limit:
            errors.append(f"{r} ends at the top of the {addr_bits}-bit address space; "
                          f"its exclusive 'last' (0x{limit:X}) does not fit in AddrWidth bits")
    check_overlaps(regions, "memory.regions", errors)
    for r in regions:
        check_tree(r, errors)
    errors += check_names(regions)
    if errors:
        print(f"{args.source}: {len(errors)} error(s), not generating:")
        for e in errors:
            print(f"  ERROR: {e}")
        return 1

    try:
        src = args.source.resolve().relative_to(REPO_ROOT)
    except ValueError:
        src = args.source
    args.output.write_text(generate(regions, addr_bits, src, pkg_name))
    print(f"Wrote {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

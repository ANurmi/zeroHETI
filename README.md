# zeroHETI

## Prerequisites
- [Bender](https://github.com/pulp-platform/bender) for dependency management.
- [Verilator](https://github.com/verilator/verilator) for RTL simulation (5.008 prefered).
- [RISC-V GNU Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain) for software compilation. Precompiled packages should suffice for now.

## Quickstart
Pull dependencies with `make ips`

Software examples are compiled with `make elf TEST=<testname>`

The Verilator model is compiled with `make verilate`. Supplying `TEST` at this stage will directly load the elf `TEST` into the design through a simulatation backdoor. The simulation is run with `make simv TEST=<testname>`. The default program load is the compile-time preload, but can be overwriten by supplying `LOAD=JTAG`

The targets can be chained into a neat one-liner: `make elf verilate simv TEST=<testname> (LOAD={JTAG})`

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added
- JTAG idcode configuration at compile-time

### Fixed
- rt-ibex interrupt timing [patch](https://github.com/soc-hub-fi/rt-ibex/pull/8)

## [v0.1.1] - 2026-08-10

### Added
- Support for real APB UART in Verilator

### Fixed
- Nested interrupt trampoline on ILP32E now stacks the full standard
  caller-saved register set (incl. `t1`, `t2` and `a4`/`x14`), fixing
  corruption of interrupted code (e.g. `compiler_builtins` `__udivdi3`)
  ([#106](https://github.com/ANurmi/zeroHETI/issues/106))

### Removed
- Local RISC-V compliance setup

## [v0.1.0] - 2026-07-03

### Added
- Initial versioning

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added
- AXI-Lite manager port to crossbar
- `uart_wrapper` module to cleaner hierarchy

## [v0.1.5] - 2026-09-01

### Added
- `obi_mb_sram_intf` support up to 16 banks

### Fixed
- Verilator lint warnings

### Removed
- HETIC interrupt controller support

## [v0.1.4] - 2026-09-01

### Fixed
- Mailbox OBI error signal X-propagation

## [v0.1.3] - 2026-08-28
- No HW updates

## [v0.1.2] - 2026-08-26

### Added
- `rt-ibex` support for dynamic priorities
- `rt-ibex` new custom CSRs for dynamic priority operation
- JTAG idcode configuration at compile-time

### Fixed
- Dynamic CPU threshold gated with config register
- `apb_mtimer` prescaler width from 3 to 10 bits
- `rt-ibex` interrupt timing [patch](https://github.com/soc-hub-fi/rt-ibex/pull/8)

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

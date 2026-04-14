# zeroHETI Control Simulation Demonstrator

## Abstract

This demonstrator simulates the operation of zeroHETI in cyber-physical control system with hard real-time constraints and as part of a larger System-on-Chip (SoC). The demonstrator is created primarily as a testbed for evaluation and comparison of the [`edfic`](https://github.com/ANurmi/edfic/tree/main) against the PULP Platform core-local interrupt controller ([`pulp-clic`](https://github.com/pulp-platform/clic/releases/tag/v2.0.0)) in a representative in-system simulation.

## Prerequisites

- TODO:bender
- TODO:verilator
- TODO:cargo
- TODO:riscvgcc

## System Description

## Task Set Description



### Block Diagram

### Mailbox Memory Map

|Name           |Description                       |Access            | Address     |
|---------------|----------------------------------|------------------|-------------|
|`MBX_STAT    ` | General status register          | OBI (R), AXI (R) | 0x0003_0000 |
|`MBX_OBI_CTRL` | zeroHETI-facing control register | OBI (RW)         | 0x0003_0004 |
|`MBX_AXI_CTRL` | SoC-facing control register      | AXI (RW)         | 0x0003_0008 |
|`MBX_IADD    ` | Inbox letter address             | AXI (W), OBI (R) | 0x0003_000C |
|`MBX_IDAT    ` | Inbox letter data                | AXI (W), OBI (R) | 0x0003_0010 |
|`MBX_OADD    ` | Outbox letter address            | AXI (R), OBI (W) | 0x0003_0014 |
|`MBX_ODAT    ` | Outbox letter data               | AXI (R), OBI (W) | 0x0003_0018 |

### Mailbox Bitfields 
- `MBX_STAT`:
    - `[3]`: Outbox full
    - `[2]`: Outbox empty
    - `[1]`: Inbox full
    - `[0]`: Inbox empty

- `MBX_OBI_CTRL`:
    - `[24]`: Inbox read acknowledge
    - `[17]`: Interrupt clear 
    - `[16]`: Interrupt set 
    - `[9]`: Flush outbox
    - `[8]`: Flush inbox
    - `[0]`: Outbox letter send


- `MBX_AXI_CTRL`:
    - `[2]`: Interrupt set
    - `[1]`: Inbox letter send
    - `[0]`: Outbox read acknowledge

## Parameters

For Rust-based builds parameters can be passed to the simulation by prefixing the appropriate environment variables to the Cargo-call, e.g.:
```
RUNTIME_MS=2 cargo run --release -Frtl-tb -Fintc-clic --example control_sim
```
Parameters passable through environment variables are:
- `LOAD_FACTOR`: TODO
- `RUNTIME_MS`: The runtime of the measured part of the application in milliseconds.

TODO: Zephyr

## Experimental Workflow

## Measured Results
# FPGA

## Workflow: set up PYNQ-Z1 to be used with OpenOCD

The FT2232 probe is connected with the following pinout:

```text
BD0 -> CK_IO37
BD1 -> CK_IO0
BD2 -> CK_IO1
BD3 -> CK_IO3
GND -> GND
```

The UART pins are mapped to:

```text
UART_RX_I -> CK_IO12
UART_TX_O -> CK_IO13
```

On-board switches are connected as follows:

- SW0: system reset
- SW1: JTAG TRST_N

To allow an OpenOCD / debugger connection, the switches must be configured as follows:

| Switch | State |
| :-:    | :-:   |
| SW1    | UP    |
| SW0    | DOWN  |

## Workflow: run a program using OpenOCD & GDB

Prerequisites:

- [PYNQ-Z1 physical setup](#workflow-set-up-pynq-z1-to-be-used-with-openocd) is done
- Bitstream is flashed on board

The repository contains a [basic OpenOCD configuration](./scripts/ft232_openocd_RT-SS.cfg). You may need to adapt one of the first lines with the correct 'adapter serial'. Identify the serial of your adapter with:

```sh
lsusb -v -d 0403:6010 2>/dev/null | grep iSerial | awk '{ print $3 }'
```

Launch OpenOCD with:

```sh
openocd -f ./scripts/ft232_openocd_RT-SS.cfg
```

You know it's working, when the terminal echoes "Ready for Remote Connections", open a new terminal and use:

```sh
riscv32-unknown-elf-gdb <Test ELF> -x ./scripts/connect-and-load.gdb
```

The `.gdb` file automates connecting GDB to the debug module and loading the ELF into the program memory.

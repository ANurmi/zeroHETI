# zeroHETI BSP

## Run an example

Make sure that the hardware is up to date:

```sh
bender update
# Verilate the hardware. INTC can be one of CLIC, EDFIC, HETIC.
make verilate INTC=$INTC
```

Run an example (from this directory)

```sh
# Run a hello world
cargo run --release -Frtl-tb --example hello

# Run with an example with a specific interrupt controller
cargo run --release -Frtl-tb -Fintc-clic --example all_irqs
```

## Features

* default-trap-print: Insert a weakly linked `DefaultHandler` interrupt handler
  that prints the mcause.code of the interrupt that was called.

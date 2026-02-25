# Motor control demo

```sh
# Run sim with CLIC
cargo run --release -Frtl-tb -Fintc-clic

# Run RTIC version with CLIC
cargo run --release -Frtl-tb -Fintc-clic -Frtic --bin motor_control_rtic
```

## Motor control details

There are 4 motors, with registers something like the following:

.status (read-only):

- read: current motor speed, returns a 4-byte transaction representing a u32
- domain: [0, 2^32] -> [0, "max speed" = ~35_000], "RPM".

.control (write-only):

- write: controls motor voltage, a 4-byte transaction to represent a u32
- In byte: [0, 2^32] -> [0, ~20_000] "mV".

Ideal: power should be mW == RPM.

If overvoltage sent -> sim will complain that about max. voltage

Internal I2C address mapping:

- 0: 31'h0, sim_en
- 1: M0 status
- 2: M0 control
- 3: M0 tune
- 4: M1 status
- 5: M1 control
- 6: M1 tune
- 7: M2 status
- 8: M2 control
- 9: M2 tune
- 10: M3 status
- 11: M3 control
- 12: M3 tune

# I2C Motor Control Demonstrator

This demonstrator simulates a synthetic system where zeroHETI controls four seperate (simulated) motors via a single I²C master peripheral.
The system receives high-level directives and sends reports via a mailbox interface to a (simulated) co-processor.

## Task Set Description

- [Periodic] 1 × High-level directive task
    - [Trigger] Mailbox interrupt input.
    - Receive high-level control values via mailbox.
    - Compute PID with new values for all motors.
    - Write to `irq_ack` when done to signal completion.

- [Sporadic] 4 × Motor speed warning tasks
    - [Trigger] Motor speed (target-to-real) delta exceeds predefined threshhold.
    - Affected by random enviromental factors.
    - Compute PID for affected motor.
    - Complete when motor speed stable again.

- [Periodic] 4 × Motor speed reporting tasks
    - [Trigger] Every XX us from test start, use timer group interrupts.
    - Read individual motor speed and timestamp atomically.
    - Write timestamp to mailbox first, then motor speed.
    - Complete when motor-specific value written to mailbox.

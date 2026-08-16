# Real-time profiling benchmark

```sh
# Run sim with CLIC
LOAD_FACTOR=0 RUNTIME_MS=10 cargo run --release -Frtl-tb -Fintc-clic
```

## Observability trace (`obs` feature)

```sh
# Run the sim with the observability trace
LOAD_FACTOR=50 RUNTIME_MS=10 cargo run --release -Frtl-tb -Fintc-clic -Fobs
```

`obs` wires the RTIC observability hooks (`on_task_act` / `on_task_comp` /
`on_res_acq` / `on_res_rel`, see `rtic-core`) to a ring-buffer backend in
`src/obs.rs`. Every task activation/completion and resource acquire/release is
recorded and dumped over UART during teardown, e.g.:

```
[obs] trace: 102 events
[obs] act   StartSim
[obs] comp  StartSim
[obs] act   Mail
[obs] acq   ibx t=254 c=254
[obs] rel   ibx t=254 c=254
...
[obs] act   swd-0xf1
[obs] act   Report1
[obs] acq   ctrl_buf_1 t=241 c=252
[obs] rel   ctrl_buf_1 t=241 c=252
```

`t` is the running task's priority and `c` the raised SRP ceiling, so
scheduling dynamics and SRP system-ceiling behavior are fully reconstructable.

The backend is deliberately lock-free: it relies on RTIC's single-hart, LIFO
preemption guarantees (a hook can only be interrupted by a strictly
higher-priority task) instead of masking interrupts. Task handlers run with
`mstatus.MIE` set and preemption gated by the CLIC level threshold, so a
critical section would perturb the timing the hooks exist to observe. Appends
commit the tail index with `max`, so a nested claim can drop at most one event
and never corrupts the buffer. See the module docs in `src/obs.rs` for the
full concurrency model.

Runtime: the hooks add only ~1.4k retired instructions / ~0.7% of active time,
but the teardown UART dump dominates wall-clock time in the sim (~25 s with
`obs` vs ~3 s without at the default `RUNTIME_MS=10`). It is disabled by
default so quick benchmark runs stay fast; without the feature, no hook code
is emitted at all.

## VS Code

For rust-analyzer on VS Code, you'll need this in settings.json:

```json
"rust-analyzer.server.extraEnv": {
    "RUNTIME_MS": 10,
    "LOAD_FACTOR": 50,
},
```
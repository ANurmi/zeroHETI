# Real-time profiling benchmark

```sh
# Run sim with CLIC
LOAD_FACTOR=0 RUNTIME_MS=10 cargo run --release -Frtl-tb -Fintc-clic
```

## VS Code

For rust-analyzer on VS Code, you'll need this in settings.json:

```json
"rust-analyzer.server.extraEnv": {
    "RUNTIME_MS": 10,
    "LOAD_FACTOR": 50,
},
```

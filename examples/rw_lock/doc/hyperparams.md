# Experimental setup & hyperparameters

Show that..?

| Condition | Intuition                                                          |
| --------- | ------------------------------------------------------------------ |
| C1        | There must be reader concurrency to exploit                        |
| C2        | The highest-priority accessor must be a reader                     |
| C3        | The task must be positioned so that writer blocking can be avoided |
| C4        | Reader blocking must actually be the dominant blocking term        |
| C5        | Removing that blocking term must change the maximum                |

## Baseline system

These hyperparams demonstrate a system that satisfies the conditions of Theorem~XYZ, and that is not schedulable using mutex, but that becomes schedulable using RW locks.

- Runtime:          45 ms
- Task periods (hyperperiod = LCD = 42 ms):
    - T(`ReaderHigh`):  100 us
    - T(`J`):           200 us
    - T(`ReaderLow`):   300 us
    - T(`Writer`):      500 us
- Interference free computation times (IRQ entry/exit overhead $<1 us$ not accounted for):
    - C(`ReaderHigh`)    10 us
    - C(`J`)             40 us
    - C(`ReaderLow`)    150 us
    - C(`Writer`)        10 us
- Critical section length (max. 30 ns jitter):
    - CS(`ReaderHigh`)   10 us
    - CS(`J`)             0 us
    - CS(`ReaderLow`)   150 us
    - CS(`Writer`)       10 us
- Deadline d(`J`):      400 us

Output:

```sh
### RW-lock schedulability demo (zeroHETI / RTIC) ###
- Timer res.   (ns) : 10
- Timer max.   (ms) : 42949
- Lock mode         : mutex
- RUNTIME_MS        : 45
- RL CS        (us) : 240
- J deadline   (us) : 400
Hyperperiod    (ms) : 42
Control::Setup
Control::Teardown
- Runtime (us): 45003
- ReaderHigh (p=0xfc): worst   208 us | n_complete   64
- J          (p=0xfb): worst   408 us | n_complete   37 | misses    3
- ReaderLow  (p=0xf9): worst   608 us | n_complete   44
- Writer     (p=0xf8): worst   625 us | n_complete   29
VERDICT [mutex]: J NOT schedulable -- 3/37 jobs missed the 400 us deadline
[TB] Program returned EXIT_SUCCESS

### RW-lock schedulability demo (zeroHETI / RTIC) ###
- Timer res.   (ns) : 10
- Timer max.   (ms) : 42949
- Lock mode         : rw-lock
- RUNTIME_MS        : 45
- RL CS        (us) : 240
- J deadline   (us) : 400
Hyperperiod    (ms) : 42
Control::Setup
Control::Teardown
- Runtime (us): 45003
- ReaderHigh (p=0xfc): worst    25 us | n_complete   64
- J          (p=0xfb): worst   392 us | n_complete   37 | misses    0
- ReaderLow  (p=0xf9): worst   608 us | n_complete   44
- Writer     (p=0xf8): worst   625 us | n_complete   29
VERDICT [rw-lock]: J schedulable -- 0/37 jobs missed the 400 us deadline
[TB] Program returned EXIT_SUCCESS
```

# CS(`Writer`) <- CS(`ReaderLow`) => C5 no longer applies

Matching locking time of `Writer` to locking time of `ReaderLow` results in condition C5 being broken.

Set CS(`Writer`): 16 us -> 240 us, CS(`ReaderLow`) == 240 us

```sh
### RW-lock schedulability demo (zeroHETI / RTIC) ###
- Timer res.   (ns) : 10
- Timer max.   (ms) : 42949
- Lock mode         : mutex
- RUNTIME_MS        : 45
- RL CS        (us) : 240
- J deadline   (us) : 400
Hyperperiod    (ms) : 42
Control::Setup
Control::Teardown
- Runtime (us): 45003
- ReaderHigh (p=0xfc): worst   249 us | n_complete   64
- J          (p=0xfb): worst   408 us | n_complete   37 | misses    3
- ReaderLow  (p=0xf9): worst   608 us | n_complete   44
- Writer     (p=0xf8): worst   849 us | n_complete   29
VERDICT [mutex]: J NOT schedulable -- 3/37 jobs missed the 400 us deadline

### RW-lock schedulability demo (zeroHETI / RTIC) ###
- Timer res.   (ns) : 10
- Timer max.   (ms) : 42949
- Lock mode         : rw-lock
- RUNTIME_MS        : 45
- RL CS        (us) : 240
- J deadline   (us) : 400
Hyperperiod    (ms) : 42
Control::Setup
Control::Teardown
- Runtime (us): 45003
- ReaderHigh (p=0xfc): worst   249 us | n_complete   64
- J          (p=0xfb): worst   392 us | n_complete   37 | misses    0
- ReaderLow  (p=0xf9): worst   608 us | n_complete   44
- Writer     (p=0xf8): worst   849 us | n_complete   29
VERDICT [rw-lock]: J schedulable -- 0/37 jobs missed the 400 us deadline
```

# CS_W > CS_RLO => C4 no longer applies

# Switch pi(J) and pi(ReaderLow) => C3 no longer applies

# pi(Writer) > pi(rest) => C2 no longer applies

# Set either reader to non-locking => C1 no longer applies

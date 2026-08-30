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
- Workload:
    - C(`ReaderHigh`)    15 us
    - C(`J`)             40 us
    - C(`ReaderLow`)     95 us
    - C(`Writer`)        15 us
- Critical section length (max. 30 ns jitter):
    - CS(`ReaderHigh`)   15 us
    - CS(`J`)             0 us
    - CS(`ReaderLow`)    95 us
    - CS(`Writer`)       15 us

Output:

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 69%                                             │- Theoretical load  : 69%
Exec                                                                  | Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 83%, instr. count: 125812                       │- True CPU util.    : 83%, instr. count: 125813
- ReaderHigh (p=0xfc): worst    86 us | n_complete   30 | misses    0 │- ReaderHigh (p=0xfc): worst    30 us | n_complete   30 | misses    0
-          J (p=0xfb): worst    87 us | n_complete   15 | misses    0 │-          J (p=0xfb): worst    76 us | n_complete   15 | misses    0
-  ReaderLow (p=0xf9): worst   166 us | n_complete   10 | misses    0 │-  ReaderLow (p=0xf9): worst   188 us | n_complete   10 | misses    0
-     Writer (p=0xf8): worst   209 us | n_complete    6 | misses    0 │-     Writer (p=0xf8): worst   209 us | n_complete    6 | misses    0
```

## Invalidate C5

- W(`Writer`) = W(`ReaderLow`) = 65 us, selected to align theoretical load with baseline system at 69%.
- => C5 no longer applies

Matching locking time of `Writer` to locking time of `ReaderLow` results in condition C5 being broken.

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 69%                                             │- Theoretical load  : 69%
Exec                                                                  | Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 83%, instr. count: 126004                       │- True CPU util.    : 83%, instr. count: 126003
- ReaderHigh (p=0xfc): worst    82 us | n_complete   30 | misses    0 │- ReaderHigh (p=0xfc): worst    82 us | n_complete   30 | misses    0
-          J (p=0xfb): worst   150 us | n_complete   15 | misses    0 │-          J (p=0xfb): worst   150 us | n_complete   15 | misses    0
-  ReaderLow (p=0xf9): worst   136 us | n_complete   10 | misses    0 │-  ReaderLow (p=0xf9): worst   195 us | n_complete   10 | misses    0
-     Writer (p=0xf8): worst   229 us | n_complete    6 | misses    0 │-     Writer (p=0xf8): worst   229 us | n_complete    6 | misses    0
```

## Invalidate C4

- W(`Writer`) = 80 us, W(`ReaderLow`) = 15 us, flipping their magnitude order
- CS_W > CS_RLO => C4 no longer applies
- Total theoretical work load is reduced

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 56%                                             │- Theoretical load  : 56%
Exec                                                                  | Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 69%, instr. count: 105522                       │- True CPU util.    : 69%, instr. count: 105522
- ReaderHigh (p=0xfc): worst    92 us | n_complete   30 | misses    0 │- ReaderHigh (p=0xfc): worst    92 us | n_complete   30 | misses    0
-          J (p=0xfb): worst    93 us | n_complete   15 | misses    0 │-          J (p=0xfb): worst    93 us | n_complete   15 | misses    0
-  ReaderLow (p=0xf9): worst    92 us | n_complete   10 | misses    0 │-  ReaderLow (p=0xf9): worst    92 us | n_complete   10 | misses    0
-     Writer (p=0xf8): worst   172 us | n_complete    6 | misses    0 │-     Writer (p=0xf8): worst   172 us | n_complete    6 | misses    0

```

## Invalidate C3

- Switch `pi(J)` and `pi(ReaderLow)` => C3 no longer applies

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 69%                                             │- Theoretical load  : 69%
Exec                                                                  | Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 83%, instr. count: 125820                       │- True CPU util.    : 83%, instr. count: 125815
- ReaderHigh (p=0xfc): worst    41 us | n_complete   30 | misses    0 │- ReaderHigh (p=0xfc): worst    29 us | n_complete   30 | misses    0
-          J (p=0xfb): worst   188 us | n_complete   15 | misses    0 │-          J (p=0xfb): worst   188 us | n_complete   15 | misses    0
-  ReaderLow (p=0xf9): worst   120 us | n_complete   10 | misses    0 │-  ReaderLow (p=0xf9): worst   142 us | n_complete   10 | misses    0
-     Writer (p=0xf8): worst   209 us | n_complete    6 | misses    0 │-     Writer (p=0xf8): worst   209 us | n_complete    6 | misses    0
```

## Invalidate C2

- `pi(Writer)` = 0xfe = 254 > `pi(rest)` => C2 no longer applies
- CS(`ReaderLow`) = 70 us
- Set periods
    - T(`ReaderHigh`)   = 200
    - T(`J`)            = 300
    - T(`ReaderLow`)    = 500
    - T(`W`)            = 100

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 49%                                             │- Theoretical load  : 49%
Exec                                                                  | Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 63%, instr. count: 96529                        │- True CPU util.    : 63%, instr. count: 96529
- ReaderHigh (p=0xfc): worst    83 us | n_complete   15 | misses    0 │- ReaderHigh (p=0xfc): worst    83 us | n_complete   15 | misses    0
-          J (p=0xfb): worst    86 us | n_complete   10 | misses    0 │-          J (p=0xfb): worst    86 us | n_complete   10 | misses    0
-  ReaderLow (p=0xf9): worst   162 us | n_complete    6 | misses    0 │-  ReaderLow (p=0xf9): worst   162 us | n_complete    6 | misses    0
-     Writer (p=0xf8): worst    82 us | n_complete   30 | misses    0 │-     Writer (p=0xf8): worst    82 us | n_complete   30 | misses    0
```

## Invalidate C1

- Set either reader to non-locking => C1 no longer applies
- Set `ReaderLow` to non-locking.

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             │Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 69%                                             │- Theoretical load  : 69%
- ReaderLow CS (us) : 95                                              │- ReaderLow CS (us) : 95
- J deadline   (us) : 200                                             │- J deadline   (us) : 200
Exec                                                                  │ Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 82%, instr. count: 124923                       │- True CPU util.    : 82%, instr. count: 124923
- ReaderHigh (p=0xfc): worst    28 us | n_complete   30 | misses    0 │- ReaderHigh (p=0xfc): worst    28 us | n_complete   30 | misses    0
-          J (p=0xfb): worst    74 us | n_complete   15 | misses    0 │-          J (p=0xfb): worst    74 us | n_complete   15 | misses    0
-  ReaderLow (p=0xf9): worst   187 us | n_complete   10 | misses    0 │-  ReaderLow (p=0xf9): worst   187 us | n_complete   10 | misses    0
-     Writer (p=0xf8): worst   208 us | n_complete    6 | misses    0 │-     Writer (p=0xf8): worst   208 us | n_complete    6 | misses    0
```

## Scenario 5 (Invalidation of C3)

- `ReaderWriter` reads for 90% of the CS, then writes for 10%.
- There is only one pure reader (`Reader`)
- There are two jobs for which C3 does not apply.

```sh
- Lock mode         : mutex                                           │- Lock mode         : rw-lock
- Pre-trigger  (us) : Some(10)                                        │- Pre-trigger  (us) : Some(10)
- Target RUNTIME_MS : 3                                               │- Target RUNTIME_MS : 3
Task set:                                                             |Task set:
- Hyperperiod  (ms) : 3                                               │- Hyperperiod  (ms) : 3
- Theoretical load  : 69%                                             │- Theoretical load  : 69%
Exec                                                                  │ Exec
- Runtime      (us) : 3000                                            │- Runtime      (us) : 3000
- True CPU util.    : 82%, instr. count: 124923                       │- True CPU util.    : 82%, instr. count: 125697
- Reader (p=0xfc)   : worst    28 us | n_complete   30 | misses     0 │- Reader (p=0xfc)   : worst    19 us | n_complete   30 | misses    0
-         J1 (p=0xfb): worst    74 us | n_complete   15 | misses    0 │-        J1 (p=0xfb): worst    65 us | n_complete   15 | misses    0
-         J2 (p=0xf9): worst   187 us | n_complete   10 | misses    0 │-        J2 (p=0xf9): worst   187 us | n_complete   10 | misses    0
- ReaderWriter (p=0xf8): worst   208 us | n_complete    6 | misses  0 │- ReaderWriter (p=0xf8): worst   275 us | n_complete 6 | misses    0
```

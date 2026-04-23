# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `8.348 us` per iteration (`119,793.12 ops/sec`)
- `planner_overhead_auto`: `0.419 us` per iteration (`2,388,938.83 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust` (`stepwise_paths`): `70.191 ms` per run (`1,424,681.54 paths/sec`)
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `1.409 ms` per run (`70,983,521.65 paths/sec`)
- `mc_cpu_european_call_numpy` (`stepwise_paths`): `101.711 ms` per run (`983,181.72 paths/sec`)
- `mc_cpu_european_call_numpy_terminal` (`terminal_distribution`): `1.350 ms` per run (`74,093,868.58 paths/sec`)
- `mc_cpu_european_call_numba` (`stepwise_paths`): `251.131 ms` per run (`398,198.31 paths/sec`)
- `mc_cpu_european_call_numba_terminal` (`terminal_distribution`): `3.659 ms` per run (`27,328,524.72 paths/sec`)

From `release-results.json`:

- `mc_cpu_european_call_rust` (`stepwise_paths`): `21.702 ms` per run (`4,607,846.60 paths/sec`)
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `0.987 ms` per run (`101,317,122.59 paths/sec`)
- `mc_cpu_european_call_numpy` (`stepwise_paths`): `158.613 ms` per run (`630,465.79 paths/sec`)
- `mc_cpu_european_call_numpy_terminal` (`terminal_distribution`): `2.336 ms` per run (`42,815,856.28 paths/sec`)
- `mc_cpu_european_call_numba` (`stepwise_paths`): `360.417 ms` per run (`277,456.52 paths/sec`)
- `mc_cpu_european_call_numba_terminal` (`terminal_distribution`): `5.170 ms` per run (`19,343,554.61 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

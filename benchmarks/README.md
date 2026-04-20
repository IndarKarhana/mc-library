# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `8.097 us` per iteration (`123,505.84 ops/sec`)
- `planner_overhead_auto`: `0.423 us` per iteration (`2,363,437.62 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust`: `1.633 ms` per run (`61,236,987.14 paths/sec`)
- `mc_cpu_european_call_numpy`: `114.334 ms` per run (`874,633.44 paths/sec`)
- `mc_cpu_european_call_numba`: `231.410 ms` per run (`432,133.83 paths/sec`)

From `release-results.json`:

- `mc_cpu_european_call_rust`: `0.575 ms` per run (`173,858,416.66 paths/sec`)
- `mc_cpu_european_call_numpy`: `112.371 ms` per run (`889,905.47 paths/sec`)
- `mc_cpu_european_call_numba`: `222.869 ms` per run (`448,695.03 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

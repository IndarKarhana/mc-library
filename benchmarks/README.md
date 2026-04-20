# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `8.380 us` per iteration (`119,336.85 ops/sec`)
- `planner_overhead_auto`: `0.415 us` per iteration (`2,408,163.48 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust`: `1.234 ms` per run (`81,004,455.25 paths/sec`)
- `mc_cpu_european_call_numpy`: `81.402 ms` per run (`1,228,476.70 paths/sec`)
- `mc_cpu_european_call_numba`: `227.355 ms` per run (`439,841.34 paths/sec`)

From `release-results.json`:

- `mc_cpu_european_call_rust`: `0.518 ms` per run (`193,153,854.77 paths/sec`)
- `mc_cpu_european_call_numpy`: `88.099 ms` per run (`1,135,090.43 paths/sec`)
- `mc_cpu_european_call_numba`: `227.294 ms` per run (`439,959.22 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

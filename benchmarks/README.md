# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `8.862 us` per iteration (`112,845.86 ops/sec`)
- `planner_overhead_auto`: `0.455 us` per iteration (`2,197,058.10 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust`: `1.838 ms` per run (`54,416,013.69 paths/sec`)
- `mc_cpu_european_call_numpy`: `109.084 ms` per run (`916,721.46 paths/sec`)
- `mc_cpu_european_call_numba`: `258.250 ms` per run (`387,222.21 paths/sec`)

From `release-results.json`:

- `mc_cpu_european_call_rust`: `0.594 ms` per run (`168,401,388.08 paths/sec`)
- `mc_cpu_european_call_numpy`: `96.824 ms` per run (`1,032,800.45 paths/sec`)
- `mc_cpu_european_call_numba`: `229.511 ms` per run (`435,709.44 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

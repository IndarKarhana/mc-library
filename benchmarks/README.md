# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `7.655 us` per iteration (`130,630.80 ops/sec`)
- `planner_overhead_auto`: `0.382 us` per iteration (`2,616,459.94 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

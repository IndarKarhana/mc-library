# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: see `latest-results.json`
- `planner_overhead_auto`: see `latest-results.json`
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust` (`stepwise_paths`): tracked as the fair CPU baseline
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): tracked as the first variance-reduced CPU path
- `mc_cpu_european_call_rust_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): tracked as the specialized fast path
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: reports `stderr_ratio_vs_standard`

From `release-results.json`:

- `mc_cpu_european_call_rust` (`stepwise_paths`): `18.700 ms` per run
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): `38.889 ms` per run
- `mc_cpu_european_call_rust_antithetic_quality`: `stderr_ratio_vs_standard = 0.747`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `0.790 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic` (`terminal_distribution_antithetic`): `1.646 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: `stderr_ratio_vs_standard = 0.741`
- `mc_cpu_european_call_numpy` (`stepwise_paths`): compare in `release-results.json`
- `mc_cpu_european_call_numba` (`stepwise_paths`): compare in `release-results.json`

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.

# mc-library

Agent-native Monte Carlo runtime with CPU, NVIDIA CUDA, and Apple Metal execution paths.

## Current Stage

This repository is in active build-out.

- architecture and design docs are in `docs/`
- project-level agent instructions are in `AGENTS.md`
- Codex project skills are in `./.codex/skills/`
- core engineering rules are in `docs/repository-rules.md`
- roadmap is in `roadmap.md`
- Rust workspace scaffolding is in `crates/`

## Core Principles

- architecture and design docs are source of truth
- roadmap is a living artifact and must be maintained
- test-driven development is the default workflow
- production-grade quality bar from day one
- fast and lightweight implementation choices

## Workspace Layout

- `crates/mc-schema`: schema types, diagnostics, compatibility, and validation
- `crates/mc-core`: planner interfaces, backend contract, CPU runtime, and execution planning
- `crates/mc-bench`: benchmark harness and benchmark result schema

## Expressive API Example

```rust
use mc_core::{EuropeanCallMethod, EuropeanCallPricer};

let result = EuropeanCallPricer::new()
    .s0(100.0)
    .strike(100.0)
    .rate(0.03)
    .volatility(0.2)
    .maturity(1.0)
    .paths(100_000)
    .steps(64)
    .seed(42)
    .method(EuropeanCallMethod::StepwisePaths)
    .price();
```

The current CPU runtime exposes both:

- a fair step-wise path benchmark path
- a specialized terminal-distribution fast path
- variance-reduction techniques including antithetic variates and control variates

The current GPU backends execute through explicit delegated CPU fallback semantics while native CUDA and Metal kernels are being built. That keeps the backend surface real and testable without overstating GPU acceleration.

`mc-core` now also exposes host-side native staging gates:

- `cuda-native`
- `metal-native`

These feature flags validate native backend boundaries, kernel metadata, and toolchain probing without requiring local GPU hardware.

The CUDA path now includes an actual staged kernel source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.cu`

When `cuda-native` is enabled and `nvcc` is available, the backend attempts PTX compilation during artifact staging and records the result in native artifact metadata. Execution still falls back to the CPU reference path until native launch support lands.

The Metal path now includes a matching staged shader source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal`

When `metal-native` is enabled and Apple developer tools are available, the backend attempts `.air` and `.metallib` compilation during artifact staging and records the result in native artifact metadata. Execution still falls back to the CPU reference path until native Metal launch support lands.

On macOS, the first Metal-native execution path is now wired through the Metal runtime itself using a staged Swift helper. That means the current Apple GPU path can execute the first step-wise European-call kernel natively on supported Macs even when standalone `xcrun metal` tooling is not available. The current implementation still uses CPU-generated normals and is a narrow v1 path, so it is a correctness-first bring-up rather than a final performance path.

## Running Tests

```bash
cargo test
```

```bash
cargo test -p mc-core --features cuda-native
```

```bash
cargo test -p mc-core --features metal-native
```

## Running Benchmarks

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

```bash
cargo run -p mc-bench --release -- --output benchmarks/release-results.json
```

## Benchmark Gates

Benchmark thresholds are documented in `docs/benchmark-gates.md`.
Competitive benchmark policy is documented in `docs/competitive-benchmark-policy.md`.
User-experience research and UX implementation plan is in `docs/user-friendliness-research.md`.
Agent integration guidance is in `docs/agent-integration-plan.md`.
Public function inventory is in `docs/function-catalog.md`.
Technique roadmap is in `docs/simulation-techniques.md`.
GPU testing strategy is in `docs/gpu-testing-strategy.md`.

## Current CPU Results

From the latest release benchmark run:

- fair step-wise Rust CPU path: `18.520 ms`
- step-wise Rust antithetic path: `35.972 ms`
- step-wise Rust control-variate path: `18.703 ms`
- step-wise NumPy baseline: see `benchmarks/release-results.json`
- step-wise Numba baseline: see `benchmarks/release-results.json`
- specialized Rust terminal-distribution fast path: `0.756 ms`
- control-variate stderr ratio vs standard:
  - step-wise: `0.411`
  - terminal: `0.412`
- antithetic stderr ratio vs standard:
  - step-wise: `0.747`
  - terminal: `0.741`

## Next Steps

- implement first NVIDIA CUDA kernels for core workload path
- implement first Apple Metal kernels for core workload path
- calibrate planner recommendations from measured backend winners once native GPU paths exist
- add Apple-specific planner heuristics for Metal-first environments
- expand competitor matrix to JAX/CuPy/PyTorch where environment allows
- extend CI from CPU validation to native hardware validation once dedicated runners exist

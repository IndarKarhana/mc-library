# Function Catalog

This document is the project-local index of important public functions and agent-callable surfaces.

## Purpose

- help contributors and agents find the right entry point quickly
- make public behavior explicit without forcing full codebase exploration
- define which surfaces are ready for future tool wrapping

## Update Rule

Update this file whenever a public function or public-facing helper is added or materially changed.

Minimum information per entry:

- path
- function or surface name
- purpose
- key inputs and outputs
- determinism or reproducibility notes
- caveats
- agent-tool readiness

## Current Public Surface

### `mc-schema`

| Surface | Path | Purpose | Inputs / Outputs | Determinism | Caveats | Agent-tool readiness |
| --- | --- | --- | --- | --- | --- | --- |
| `SimulationSpecBuilder` | `crates/mc-schema/src/builder.rs` | Ergonomic builder for `SimulationSpec` construction. | Inputs: chained builder calls. Output: `SimulationSpec`. | Deterministic. | Builder convenience layer, not execution logic. | Good candidate for future language bindings, not yet a standalone agent tool. |
| `validate_simulation_spec` | `crates/mc-schema/src/validate.rs` | Validate a simulation definition and emit structured diagnostics. | Input: `&SimulationSpec`. Output: `Vec<Diagnostic>`. | Deterministic. | Current coverage is schema-level, not full execution feasibility. | Strong candidate for an agent validation tool. |
| `check_schema_compatibility` | `crates/mc-schema/src/compat.rs` | Check schema version support and compatibility expectations. | Input: schema version metadata. Output: `CompatibilityReport`. | Deterministic. | Focused on version compatibility, not semantic correctness. | Good candidate for an agent compatibility tool. |

### `mc-core` planner

| Surface | Path | Purpose | Inputs / Outputs | Determinism | Caveats | Agent-tool readiness |
| --- | --- | --- | --- | --- | --- | --- |
| `normalize_run_config` | `crates/mc-core/src/planner/mod.rs` | Normalize and validate runtime execution settings. | Input: `RunConfig`. Output: `Result<NormalizedRunConfig, PlannerError>`. | Deterministic. | Only validates current planner config rules. | Useful helper, but better wrapped inside higher-level planning tools. |
| `extract_features` | `crates/mc-core/src/planner/mod.rs` | Compute structural features from a `SimulationSpec`. | Input: `&SimulationSpec`. Output: `FeatureSummary`. | Deterministic. | Feature set is currently v1-focused. | Good candidate for an agent analysis tool. |
| `plan_execution` | `crates/mc-core/src/planner/mod.rs` | Choose backend and produce `ExecutionPlan` with explainability. | Inputs: `&SimulationSpec`, `RunConfig`, backend support reports. Output: `Result<ExecutionPlan, PlannerError>`. | Deterministic for same inputs and support reports. | Current heuristics are still evolving, especially for GPU paths. | High-value candidate for an agent planning tool. |

### `mc-core` backend and runtime

| Surface | Path | Purpose | Inputs / Outputs | Determinism | Caveats | Agent-tool readiness |
| --- | --- | --- | --- | --- | --- | --- |
| `builtin_backends` | `crates/mc-core/src/backend/mod.rs` | Build the built-in backend registry. | No inputs. Output: `Vec<Box<dyn RuntimeBackend>>`. | Deterministic aside from runtime device discovery results. | Discovery depends on host environment. | Internal support surface, usually not a direct tool endpoint. |
| `plan_gpu_chunking` | `crates/mc-core/src/backend/mod.rs` | Estimate chunking strategy for GPU execution under memory budgets. | Inputs: total paths, device memory, `GpuChunkingConfig`. Output: `GpuChunkingPlan`. | Deterministic. | Planning heuristic only; not proof of runtime support. | Strong future tool candidate for backend explainability. |
| `estimate_gpu_bytes_per_path` | `crates/mc-core/src/backend/mod.rs` | Conservative per-path memory estimate for GPU cost modeling. | Input: `&ExecutionPlan`. Output: `usize`. | Deterministic. | Heuristic estimate, not measured runtime telemetry. | Internal helper unless exposed with explanation context. |
| `european_call_price_mc_cpu` | `crates/mc-core/src/runtime/cpu.rs` | Execute the current CPU reference workload for a European call under GBM. | Input: `&EuropeanCallConfig`. Output: `EuropeanCallResult`. | Deterministic for identical config and seed. | Narrow workload path, not the future general runtime entry point. | Good benchmark and reference tool candidate, but too workload-specific for the final public tool layer. |
| `MonteCarloRng::new` / `standard_normal` | `crates/mc-core/src/runtime/cpu.rs` | Deterministic RNG used by the current CPU reference runtime. | Input: seed or method call. Output: RNG state or sample. | Deterministic. | Internal RNG choice may change as runtime broadens. | Not a primary tool surface. |

### `mc-bench`

| Surface | Path | Purpose | Inputs / Outputs | Determinism | Caveats | Agent-tool readiness |
| --- | --- | --- | --- | --- | --- | --- |
| `run_default_benchmarks` | `crates/mc-bench/src/harness.rs` | Run the default benchmark suite and produce a structured report. | No inputs. Output: `BenchmarkReport`. | Mostly deterministic in structure, but timing values vary by machine state. | Environment-sensitive and may mark competitors unavailable. | High-value CI and agent audit tool candidate. |
| `build_competitiveness_plan` | `crates/mc-bench/src/harness.rs` | Translate benchmark results into an improvement plan. | Input: `&BenchmarkReport`. Output: markdown string. | Deterministic for same report. | Plan quality depends on benchmark coverage. | Good support tool for automated performance review. |

## Next Catalog Expansion

As the library grows, add catalog entries for:

- Python binding entry points
- explainability helpers such as plan explanation and run manifests
- future general runtime execute APIs
- tool manifest exporters or JSON-schema emitters

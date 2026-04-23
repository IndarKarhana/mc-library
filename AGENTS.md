# AGENTS.md

Project-wide operating instructions for Codex and compatible agent workflows.

## Mission

Build a production-grade, agent-native Monte Carlo runtime that is:

- faster than mainstream baseline libraries on targeted workloads
- honest about where it is not yet faster
- reproducible and explainable
- easy for both human users and AI agents to inspect and operate

This repository is not just a pricing helper. It is an execution runtime with:

- structured simulation definitions
- explicit planning and backend selection
- CPU, NVIDIA, and Apple execution paths
- benchmark-backed performance claims
- machine-readable outputs and metadata

## Mandatory Reads Before Meaningful Work

Read these before planning or implementing non-trivial changes:

1. `docs/architecture-plan.md`
2. `docs/backend-contract.md`
3. `docs/repository-rules.md`
4. `roadmap.md`

Read these when relevant:

- `docs/user-friendliness-research.md` for UX-facing changes
- `docs/competitive-benchmark-policy.md` and `benchmarks/README.md` for performance work
- `docs/planner-design.md` for planner changes
- `docs/function-catalog.md` when adding or changing public functions
- `docs/agent-integration-plan.md` when changing agent-facing surfaces

## Non-Negotiable Working Rules

1. Architecture docs are source of truth.
2. `roadmap.md` must reflect meaningful scope or status changes.
3. TDD is the default: write or update failing tests first when practical.
4. Production-grade quality is required even for early infrastructure.
5. Hot-path work must remain lightweight and benchmark-conscious.
6. Do not claim performance wins without measured evidence.
7. Do not hide unsupported behavior behind silent fallback.
8. Prefer structured diagnostics and manifests over opaque strings.

## Project-Specific Engineering Standards

### Numerical and statistical standards

- Treat the CPU backend as the current correctness reference.
- Distinguish exact reproducibility from statistical reproducibility.
- Document mathematical assumptions, estimator behavior, and reduction semantics.
- Prefer deterministic stream partitioning and stable reduction order where feasible.

### Performance standards

- Optimize the narrow fast path first.
- Avoid avoidable allocations, virtual overhead, and unnecessary dependencies on hot paths.
- When touching runtime, planner cost modeling, or benchmark harnesses, run relevant benchmarks.
- If a benchmark regresses or loses to a competitor, update the improvement plan.

### API and agent standards

- Public APIs must be typed, predictable, and inspectable.
- Public functions and agent-callable entry points must be documented in `docs/function-catalog.md`.
- Favor machine-readable structs and enums over free-form output.
- New agent-facing surfaces should define purpose, inputs, outputs, determinism, and failure modes explicitly.
- Keep plan, compile, execute, and explain responsibilities separated.

## Function Catalog Rule

Whenever you add or materially change a public function, update `docs/function-catalog.md` with:

- function name
- file path
- purpose
- key inputs and outputs
- determinism or reproducibility notes
- important errors or caveats
- whether the function is suitable as an agent tool surface

If the function is not yet fit for agent use, say why.

## Definition Of Done

Work is done when all of the following are true:

1. Implementation is complete.
2. Relevant tests pass.
3. Docs are updated.
4. `roadmap.md` is updated when status or scope changed.
5. Benchmarks are updated when hot-path performance changed.
6. Known limitations or follow-up work are documented honestly.

## Preferred Build Direction

When tradeoffs are unclear, bias toward:

- explicit over implicit
- deterministic over magical
- measurable over assumed
- narrow fast paths over premature generality
- agent-readable interfaces over callback-heavy hidden behavior

The goal is a library that a senior quant, a production engineer, and an autonomous coding agent can all trust.

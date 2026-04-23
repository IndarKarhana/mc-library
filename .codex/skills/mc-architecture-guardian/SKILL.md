---
name: mc-architecture-guardian
description: Keep Monte Carlo runtime work aligned with architecture, roadmap, benchmarking discipline, and production-grade quality.
metadata:
  short-description: Architecture and quality guardrails for mc-library
---

# mc-architecture-guardian

Use this skill when planning, implementing, or reviewing non-trivial work in this repository.

TRIGGER when: a change touches planner, runtime, backend, schema, benchmarks, public API, roadmap, or project structure.

## Required reads

Read these before making substantial changes:

- `AGENTS.md`
- `docs/architecture-plan.md`
- `docs/backend-contract.md`
- `docs/repository-rules.md`
- `roadmap.md`

Load these when relevant:

- `docs/planner-design.md`
- `docs/user-friendliness-research.md`
- `docs/competitive-benchmark-policy.md`
- `docs/function-catalog.md`
- `docs/agent-integration-plan.md`

## Operating rules

1. Treat architecture docs as authoritative unless you are intentionally updating them.
2. Keep roadmap state current with real implementation progress.
3. Use TDD by default for logic-heavy changes.
4. Keep runtime and planner surfaces explicit and machine-readable.
5. Do not hide unsupported backend behavior behind silent fallback.
6. Keep hot-path code lightweight and benchmark-aware.

## Execution checklist

1. Identify which architecture invariants the change touches.
2. Write or update tests first when practical.
3. Implement the smallest change that preserves the documented design.
4. If public APIs changed, update `docs/function-catalog.md`.
5. If performance-sensitive code changed, run relevant benchmarks and update benchmark artifacts if needed.
6. Update `roadmap.md` and any affected docs before closing the task.

## Review lens

Ask these questions before finishing:

- Is the behavior explicit and reproducible?
- Is the backend capability honest?
- Is the API easy for an agent to inspect and call?
- Are performance claims backed by actual measurements?
- Would a future contributor know where to extend this safely?

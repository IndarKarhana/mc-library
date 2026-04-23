# Agent Integration Plan

This document defines how the library should evolve so AI agents can use it safely, predictably, and efficiently.

## Objective

Make the library easy to wrap as tools without forcing agents to reverse-engineer the codebase.

The target state is:

- clear public entry points
- typed request and response objects
- explicit determinism and backend-support semantics
- machine-readable planning and execution outputs
- a stable catalog of callable surfaces

## Principles

1. Structured over magical
- Tool-facing APIs should prefer typed configs and result structs.

2. Explainability is part of the API
- If the planner selects a backend, chunking strategy, or fallback, the reason should be inspectable.

3. Unsupported is a first-class outcome
- Agents should get explicit unsupported states, not silent fallback.

4. Narrow fast path first
- Start with a small set of highly reliable tool surfaces before broadening.

## Recommended Tool Surface Layers

### Layer 1: Analysis

Safe, deterministic surfaces that inspect inputs without running the full runtime.

Examples:

- schema validation
- schema compatibility checks
- feature extraction
- execution planning
- benchmark report analysis

### Layer 2: Reference execution

Narrow execution surfaces with deterministic seed handling and explicit workload scope.

Examples:

- CPU European call reference execution
- future explain-plan helper

### Layer 3: General runtime execution

Higher-level runtime calls once backend contracts, manifests, and cross-backend guarantees are mature.

## Near-Term Deliverables

1. Keep `docs/function-catalog.md` current.
2. Add a stable explainability helper around `ExecutionPlan`.
3. Add a machine-readable run manifest format for runtime outputs.
4. Add Python-facing wrappers that preserve the same structured semantics.
5. Add a tool manifest or JSON-schema export for agent wrappers once the public surface stabilizes.

Current status:

- `explain_execution_plan` exists as the first lightweight planner explanation helper
- structured run manifests are still pending

## Tool-Readiness Checklist

A surface is ready to be wrapped as an agent tool when:

- inputs are typed and explicit
- outputs are structured
- errors are actionable
- determinism expectations are documented
- unsupported states are explicit
- source location is documented in `docs/function-catalog.md`
- tests cover the contract

## What We Should Avoid

- global implicit state
- callback-heavy execution APIs
- planner decisions that cannot be explained
- backend behavior that changes silently by environment
- free-form text as the only result channel

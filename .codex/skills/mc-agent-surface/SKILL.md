---
name: mc-agent-surface
description: Design and document public functions and tool-friendly interfaces so agents can call the library safely and predictably.
metadata:
  short-description: Agent-friendly API and function-catalog discipline
---

# mc-agent-surface

Use this skill when adding or changing public functions, manifests, explainability helpers, or future tool/plugin integration points.

TRIGGER when: a public function is added, a public function signature changes, a result object changes, or a new tool-facing surface is introduced.

## Primary objective

Make the library easy for agents to use without reading the whole codebase.

That means each important callable surface should have:

- a stable name
- a clear purpose
- typed inputs and outputs
- deterministic or statistical behavior notes
- explicit failure and unsupported-mode behavior
- a documented source location

## Required updates

When this skill applies, update:

1. `docs/function-catalog.md`
2. Any user-facing docs affected by the API change
3. `roadmap.md` if the work changes scope or progress

## Design rules

1. Prefer structured request and result objects over long argument lists.
2. Keep planner, compile, execute, benchmark, and explain surfaces separate.
3. Expose reproducibility metadata wherever stochastic execution occurs.
4. Return unsupported or unavailable states explicitly.
5. Avoid agent-hostile APIs that require hidden global state or implicit callbacks.

## Function-catalog entry rules

Document for each function:

- path
- signature or primary struct inputs
- what it does
- what it returns
- determinism notes
- major caveats
- whether it is ready to be wrapped as an agent tool

If not ready, state the missing pieces plainly.

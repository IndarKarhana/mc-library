# Agent Tools

The Python package exposes stable, JSON-serializable agent wrappers:

```python
from mc_library import agent_execute, agent_plan, agent_tool_manifest

print(agent_tool_manifest())

plan = agent_plan({
    "workload": "european_call",
    "config": {"n_paths": 128, "n_steps": 4, "seed": 5}
})

run = agent_execute({
    "workload": "european_call",
    "config": {"n_paths": 128, "n_steps": 4, "seed": 5}
})
```

Every response contains either:

- `ok=true` plus structured payload and manifest, or
- `ok=false` plus structured diagnostics and manifest.

See:

- `docs/agent-tooling.md`
- `docs/agent-examples.json`


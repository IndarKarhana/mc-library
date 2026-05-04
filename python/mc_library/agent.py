"""Agent-safe tool surfaces for mc-library.

The functions in this module accept and return JSON-serializable dictionaries.
They avoid hidden global state, keep unsupported behavior explicit, and attach
reproducibility metadata to planning, execution, benchmark, and reproduction
responses.
"""

from __future__ import annotations

import platform
import sys
from dataclasses import asdict
from importlib import metadata
from typing import Any, Mapping

from .benchmarks import run_benchmarks
from .methods import recommend_method
from .pricing import (
    ArithmeticAsianCallConfig,
    DownAndOutCallConfig,
    EuropeanCallConfig,
    McConfigurationError,
    price_arithmetic_asian_call,
    price_down_and_out_call,
    price_european_call,
    price_european_call_greeks,
)

SUPPORTED_WORKLOADS = {
    "european_call",
    "arithmetic_asian_call",
    "down_and_out_call",
    "european_call_greeks",
}


def agent_tool_manifest() -> dict[str, Any]:
    """Return the stable machine-readable agent tool manifest."""

    tools = [
        _tool(
            "mc.validate",
            "Validate a supported workload request without running simulation.",
            "mc.validate.request",
            "mc.validate.response",
            deterministic=True,
        ),
        _tool(
            "mc.recommend",
            "Recommend a method, sampling policy, and variance-reduction technique.",
            "mc.recommend.request",
            "mc.recommend.response",
            deterministic=True,
        ),
        _tool(
            "mc.plan",
            "Build a deterministic dry-run execution plan with cost and caveat metadata.",
            "mc.plan.request",
            "mc.plan.response",
            deterministic=True,
        ),
        _tool(
            "mc.execute",
            "Execute a narrow Python reference workload with reproducibility metadata.",
            "mc.execute.request",
            "mc.execute.response",
            deterministic=True,
        ),
        _tool(
            "mc.compare",
            "Compare fast and accuracy-oriented method choices for a workload.",
            "mc.compare.request",
            "mc.compare.response",
            deterministic=True,
        ),
        _tool(
            "mc.benchmark",
            "Return benchmark command metadata by default, or run benchmarks when explicitly requested.",
            "mc.benchmark.request",
            "mc.benchmark.response",
            deterministic=False,
        ),
        _tool(
            "mc.reproduce",
            "Create a reproduction recipe from an agent run manifest.",
            "mc.reproduce.request",
            "mc.reproduce.response",
            deterministic=True,
        ),
    ]
    return {
        "schema_version": "agent-tools.v1",
        "package": "mc_library",
        "tools": tools,
    }


def export_json_schemas() -> dict[str, dict[str, Any]]:
    """Export stable JSON-schema-like contracts for agent tools."""

    request_schema = {
        "type": "object",
        "required": ["workload"],
        "properties": {
            "workload": {"type": "string", "enum": sorted(SUPPORTED_WORKLOADS)},
            "config": {"type": "object"},
            "preferences": {"type": "object"},
        },
        "additionalProperties": False,
    }
    response_schema = {
        "type": "object",
        "required": ["ok", "manifest"],
        "properties": {
            "ok": {"type": "boolean"},
            "result": {"type": "object"},
            "manifest": _manifest_schema(),
            "diagnostics": {"type": "array", "items": {"type": "object"}},
            "reproduction": {"type": "object"},
        },
    }
    benchmark_request_schema = {
        "type": "object",
        "properties": {
            "profile": {"type": "string", "enum": ["compact", "full"]},
            "release": {"type": "boolean"},
            "execute": {"type": "boolean"},
            "repo_root": {"type": "string"},
        },
        "additionalProperties": False,
    }

    schemas = {
        "mc.validate.request": request_schema,
        "mc.validate.response": response_schema,
        "mc.recommend.request": request_schema,
        "mc.recommend.response": response_schema,
        "mc.plan.request": request_schema,
        "mc.plan.response": response_schema,
        "mc.execute.request": request_schema,
        "mc.execute.response": response_schema,
        "mc.compare.request": request_schema,
        "mc.compare.response": response_schema,
        "mc.benchmark.request": benchmark_request_schema,
        "mc.benchmark.response": response_schema,
        "mc.reproduce.request": {
            "type": "object",
            "required": ["manifest"],
            "properties": {"manifest": _manifest_schema()},
            "additionalProperties": False,
        },
        "mc.reproduce.response": response_schema,
    }
    return schemas


def agent_validate(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    config = _config_payload(request)
    diagnostics = _validate_payload(workload, config)
    ok = not diagnostics
    return {
        "ok": ok,
        "diagnostics": diagnostics,
        "manifest": _agent_manifest(
            tool="mc.validate",
            workload=workload or "unknown",
            config=config,
            method="validation_only",
            warnings=tuple(item["message"] for item in diagnostics),
        ),
    }


def agent_recommend(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    preferences = dict(request.get("preferences") or {})
    recommendation = recommend_method(
        workload_family=_recommendation_workload(workload),
        n_paths=int(config.get("n_paths", 100_000)),
        n_steps=int(config.get("n_steps", 64)),
        prefer_accuracy=bool(preferences.get("prefer_accuracy", False)),
        allow_slower_structured_sampling=bool(
            preferences.get("allow_slower_structured_sampling", False)
        ),
    )
    return {
        "ok": True,
        "recommendation": asdict(recommendation),
        "manifest": _agent_manifest(
            tool="mc.recommend",
            workload=workload,
            config=config,
            method=recommendation.method_id,
            warnings=recommendation.caveats,
        ),
    }


def agent_plan(request: Mapping[str, Any]) -> dict[str, Any]:
    recommended = agent_recommend(request)
    if not recommended["ok"]:
        return recommended

    workload = str(request["workload"])
    config = _config_payload(request)
    recommendation = recommended["recommendation"]
    plan = {
        "dry_run": True,
        "workload": workload,
        "backend": "python_reference",
        "method_id": recommendation["method_id"],
        "sampling": recommendation["sampling"],
        "technique": recommendation["technique"],
        "estimated_cost": {
            "confidence": "low",
            "estimated_runtime_ms": None,
            "estimated_peak_memory_mb": _estimate_memory_mb(config),
        },
        "rejected_methods": _rejected_methods(workload, recommendation["method_id"]),
        "notes": [
            "Dry-run planning does not execute simulation.",
            "Python reference backend is selected for stable agent examples; Rust benchmark artifacts carry performance claims.",
        ],
    }
    return {
        "ok": True,
        "plan": plan,
        "manifest": _agent_manifest(
            tool="mc.plan",
            workload=workload,
            config=config,
            method=recommendation["method_id"],
            warnings=tuple(recommendation["caveats"]),
        ),
    }


def agent_execute(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    try:
        if workload == "european_call":
            result = price_european_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "black_scholes_european_call_atm_1y"
        elif workload == "arithmetic_asian_call":
            result = price_arithmetic_asian_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "no_trusted_fixture_yet"
        elif workload == "down_and_out_call":
            result = price_down_and_out_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "no_trusted_fixture_yet"
        elif workload == "european_call_greeks":
            report = price_european_call_greeks(**config)
            payload = {
                "base_price": report.base_price,
                "greeks": dict(report.greeks),
                "explanation": report.explain(),
            }
            recipe = report.reproduce()
            reference = "black_scholes_european_call_greeks_atm_1y"
        else:
            raise AssertionError("validated workload should be supported")
    except McConfigurationError as exc:
        return _error_response("mc.execute", workload, config, exc)

    manifest = _agent_manifest(
        tool="mc.execute",
        workload=workload,
        config=config,
        method="python_reference",
        estimator="black_scholes_closed_form" if workload == "european_call_greeks" else None,
        warnings=tuple(payload.get("warnings", ())),
        reference=reference,
    )
    return {
        "ok": True,
        "result": payload,
        "manifest": manifest,
        "reproduction": {
            "python": recipe.python,
            "manifest": dict(recipe.manifest),
        },
    }


def agent_compare(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    fast = agent_recommend(
        {
            "workload": workload,
            "config": config,
            "preferences": {"prefer_accuracy": False},
        }
    )
    accurate = agent_recommend(
        {
            "workload": workload,
            "config": config,
            "preferences": {
                "prefer_accuracy": True,
                "allow_slower_structured_sampling": True,
            },
        }
    )
    alternatives = [
        {"label": "fast_default", **fast.get("recommendation", {})},
        {"label": "accuracy_oriented", **accurate.get("recommendation", {})},
    ]
    return {
        "ok": True,
        "alternatives": alternatives,
        "manifest": _agent_manifest(
            tool="mc.compare",
            workload=workload,
            config=config,
            method="method_comparison",
            warnings=(
                "Comparison is planner-policy based; use benchmark artifacts before making performance claims.",
            ),
        ),
    }


def agent_benchmark(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    profile = str(payload.get("profile", "compact"))
    release = bool(payload.get("release", False))
    execute = bool(payload.get("execute", False))
    repo_root = str(payload.get("repo_root", "."))
    command = _benchmark_command(profile, release)
    manifest = _agent_manifest(
        tool="mc.benchmark",
        workload="benchmark_suite",
        config={"profile": profile, "release": release, "execute": execute},
        method="benchmark_harness",
        warnings=(
            "Benchmark timing is environment-sensitive.",
            "Compact profile is for smoke checks, not competitiveness claims.",
        ),
    )
    if not execute:
        return {
            "ok": True,
            "status": "dry_run",
            "command": " ".join(command),
            "manifest": manifest,
        }

    results = run_benchmarks(repo_root=repo_root, release=release, profile=profile)  # type: ignore[arg-type]
    return {
        "ok": True,
        "status": "executed",
        "results": [asdict(row) for row in results],
        "manifest": manifest,
    }


def agent_reproduce(request: Mapping[str, Any]) -> dict[str, Any]:
    manifest = dict(request.get("manifest") or {})
    workload = str(manifest.get("workload", ""))
    config = dict(manifest.get("config") or {})
    helper = {
        "european_call": "price_european_call",
        "arithmetic_asian_call": "price_arithmetic_asian_call",
        "down_and_out_call": "price_down_and_out_call",
        "european_call_greeks": "price_european_call_greeks",
    }.get(workload)
    if helper is None:
        return {
            "ok": False,
            "diagnostics": [
                {
                    "code": "MC_AGENT_REPRODUCE_UNSUPPORTED",
                    "message": f"Cannot reproduce workload {workload!r}",
                    "suggestion": "Pass a manifest produced by mc.execute for a supported workload.",
                }
            ],
            "manifest": _agent_manifest(
                tool="mc.reproduce",
                workload=workload or "unknown",
                config=config,
                method="reproduce",
            ),
        }

    return {
        "ok": True,
        "reproduction": {
            "python": (
                f"from mc_library import {helper}\n"
                f"result = {helper}(**{config!r})\n"
                "print(result)\n"
            ),
            "manifest": manifest,
        },
        "manifest": _agent_manifest(
            tool="mc.reproduce",
            workload=workload,
            config=config,
            method="reproduce",
            reference=manifest.get("reference"),
        ),
    }


def _tool(
    name: str,
    description: str,
    input_schema: str,
    output_schema: str,
    *,
    deterministic: bool,
) -> dict[str, Any]:
    return {
        "name": name,
        "description": description,
        "input_schema": input_schema,
        "output_schema": output_schema,
        "determinism": "deterministic" if deterministic else "environment_sensitive",
        "failure_mode": "structured diagnostics with ok=false",
    }


def _manifest_schema() -> dict[str, Any]:
    return {
        "type": "object",
        "required": ["schema_version", "tool", "workload", "backend", "seed"],
        "properties": {
            "schema_version": {"type": "string"},
            "tool": {"type": "string"},
            "workload": {"type": "string"},
            "seed": {"type": ["integer", "null"]},
            "backend": {"type": "string"},
            "method": {"type": ["string", "null"]},
            "estimator": {"type": ["string", "null"]},
            "config": {"type": "object"},
            "build": {"type": "object"},
            "hardware": {"type": "object"},
            "warnings": {"type": "array", "items": {"type": "string"}},
            "reference": {"type": ["string", "null"]},
        },
    }


def _config_payload(request: Mapping[str, Any]) -> dict[str, Any]:
    return dict(request.get("config") or {})


def _validate_payload(workload: str, config: Mapping[str, Any]) -> list[dict[str, str]]:
    if workload not in SUPPORTED_WORKLOADS:
        return [
            {
                "code": "MC_AGENT_UNSUPPORTED_WORKLOAD",
                "message": f"Unsupported workload {workload!r}",
                "suggestion": f"Use one of: {', '.join(sorted(SUPPORTED_WORKLOADS))}",
            }
        ]
    try:
        if workload in {"european_call", "european_call_greeks"}:
            EuropeanCallConfig(**config)
        elif workload == "arithmetic_asian_call":
            ArithmeticAsianCallConfig(**config)
        elif workload == "down_and_out_call":
            DownAndOutCallConfig(**config)
    except TypeError as exc:
        return [
            {
                "code": "MC_AGENT_CONFIG_SHAPE",
                "message": str(exc),
                "suggestion": "Use documented config keys for the selected workload.",
            }
        ]
    diagnostics: list[dict[str, str]] = []
    if int(config.get("n_paths", 100_000)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_PATHS",
                "message": "n_paths must be greater than zero",
                "suggestion": "Set n_paths to a positive integer.",
            }
        )
    if int(config.get("n_steps", 64)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_STEPS",
                "message": "n_steps must be greater than zero",
                "suggestion": "Set n_steps to a positive integer.",
            }
        )
    for name in ("spot", "strike", "maturity"):
        if float(config.get(name, 1.0 if name == "maturity" else 100.0)) <= 0.0:
            diagnostics.append(
                {
                    "code": "MC_CONFIG_POSITIVE",
                    "message": f"{name} must be greater than zero",
                    "suggestion": f"Set {name} to a positive number.",
                }
            )
    if float(config.get("volatility", 0.2)) < 0.0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_VOLATILITY",
                "message": "volatility must be non-negative",
                "suggestion": "Set volatility to zero or a positive decimal.",
            }
        )
    if workload == "down_and_out_call":
        barrier = float(config.get("barrier", 80.0))
        spot = float(config.get("spot", 100.0))
        if barrier <= 0.0 or barrier >= spot:
            diagnostics.append(
                {
                    "code": "MC_CONFIG_BARRIER",
                    "message": "barrier must be positive and below spot",
                    "suggestion": "Use 0 < barrier < spot.",
                }
            )
    return diagnostics


def _recommendation_workload(workload: str) -> str:
    if workload == "european_call_greeks":
        return "european_call"
    return workload


def _estimate_memory_mb(config: Mapping[str, Any]) -> float:
    paths = int(config.get("n_paths", 100_000))
    steps = int(config.get("n_steps", 64))
    return round(paths * max(steps, 1) * 8 / (1024 * 1024), 3)


def _rejected_methods(workload: str, selected_method: str) -> list[dict[str, str]]:
    rejected = []
    if selected_method != "control_variates":
        rejected.append(
            {
                "method_id": "control_variates",
                "reason": "not selected under current accuracy preferences",
            }
        )
    if workload != "arithmetic_asian_call":
        rejected.append(
            {
                "method_id": "multilevel_monte_carlo",
                "reason": "current MLMC recommendation surface is arithmetic Asian focused",
            }
        )
    return rejected


def _pricing_payload(result: Any) -> dict[str, Any]:
    return {
        "price": result.price,
        "stderr": result.stderr,
        "explanation": result.explain(),
        "warnings": list(result.warnings),
    }


def _error_response(
    tool: str,
    workload: str,
    config: Mapping[str, Any],
    error: McConfigurationError,
) -> dict[str, Any]:
    return {
        "ok": False,
        "diagnostics": [
            {
                "code": error.code,
                "message": error.message,
                "suggestion": error.suggestion,
            }
        ],
        "manifest": _agent_manifest(
            tool=tool,
            workload=workload,
            config=config,
            method="failed_validation",
            warnings=(error.message,),
        ),
    }


def _agent_manifest(
    *,
    tool: str,
    workload: str,
    config: Mapping[str, Any],
    method: str | None,
    estimator: str | None = None,
    warnings: tuple[str, ...] = (),
    reference: Any = None,
) -> dict[str, Any]:
    return {
        "schema_version": "agent-run.v1",
        "tool": tool,
        "workload": workload,
        "seed": config.get("seed"),
        "backend": "python_reference",
        "method": method,
        "estimator": estimator,
        "config": dict(config),
        "build": {
            "package": "mc_library",
            "version": _package_version(),
            "python": sys.version.split()[0],
        },
        "hardware": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        },
        "warnings": list(warnings),
        "reference": reference,
        "determinism": "deterministic for same config, seed, package version, and Python version unless tool notes say environment-sensitive",
    }


def _package_version() -> str:
    try:
        return metadata.version("mc-library")
    except metadata.PackageNotFoundError:
        return "editable-or-source-tree"


def _benchmark_command(profile: str, release: bool) -> list[str]:
    command = ["cargo", "run", "-p", "mc-bench"]
    if release:
        command.append("--release")
    command.extend(["--", "--profile", profile])
    return command

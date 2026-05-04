import unittest

from mc_library import (
    agent_benchmark,
    agent_compare,
    agent_execute,
    agent_plan,
    agent_recommend,
    agent_reproduce,
    agent_tool_manifest,
    agent_validate,
    export_json_schemas,
)


class AgentSurfaceTests(unittest.TestCase):
    def test_tool_manifest_lists_stable_agent_tools(self) -> None:
        manifest = agent_tool_manifest()
        tool_names = {tool["name"] for tool in manifest["tools"]}

        self.assertEqual(manifest["schema_version"], "agent-tools.v1")
        self.assertIn("mc.validate", tool_names)
        self.assertIn("mc.recommend", tool_names)
        self.assertIn("mc.plan", tool_names)
        self.assertIn("mc.execute", tool_names)
        self.assertIn("mc.compare", tool_names)
        self.assertIn("mc.benchmark", tool_names)
        self.assertIn("mc.reproduce", tool_names)

    def test_json_schema_export_contains_execute_contract(self) -> None:
        schemas = export_json_schemas()

        self.assertIn("mc.execute.request", schemas)
        self.assertIn("mc.execute.response", schemas)
        self.assertEqual(schemas["mc.execute.request"]["type"], "object")
        self.assertIn("workload", schemas["mc.execute.request"]["required"])

    def test_validate_reports_supported_and_unsupported_states(self) -> None:
        ok = agent_validate({"workload": "european_call", "config": {"n_paths": 128}})
        unsupported = agent_validate({"workload": "american_call", "config": {}})
        bad_config = agent_validate({"workload": "european_call", "config": {"n_paths": 0}})

        self.assertTrue(ok["ok"])
        self.assertFalse(unsupported["ok"])
        self.assertEqual(unsupported["diagnostics"][0]["code"], "MC_AGENT_UNSUPPORTED_WORKLOAD")
        self.assertFalse(bad_config["ok"])
        self.assertEqual(bad_config["diagnostics"][0]["code"], "MC_CONFIG_PATHS")

    def test_recommend_and_plan_are_deterministic_dry_run_surfaces(self) -> None:
        request = {
            "workload": "arithmetic_asian_call",
            "config": {"n_paths": 100_000, "n_steps": 64, "seed": 7},
            "preferences": {"prefer_accuracy": True},
        }
        recommendation = agent_recommend(request)
        plan = agent_plan(request)

        self.assertTrue(recommendation["ok"])
        self.assertEqual(recommendation["recommendation"]["method_id"], "multilevel_monte_carlo")
        self.assertTrue(plan["ok"])
        self.assertTrue(plan["plan"]["dry_run"])
        self.assertEqual(plan["manifest"]["seed"], 7)
        self.assertEqual(plan["manifest"]["backend"], "python_reference")

    def test_execute_returns_structured_manifest_and_reproduction_recipe(self) -> None:
        response = agent_execute(
            {"workload": "european_call", "config": {"n_paths": 256, "n_steps": 8, "seed": 19}}
        )

        self.assertTrue(response["ok"])
        self.assertEqual(response["manifest"]["schema_version"], "agent-run.v1")
        self.assertEqual(response["manifest"]["seed"], 19)
        self.assertEqual(response["manifest"]["backend"], "python_reference")
        self.assertIn("hardware", response["manifest"])
        self.assertIn("build", response["manifest"])
        self.assertIn("price", response["result"])
        self.assertIn("price_european_call", response["reproduction"]["python"])

    def test_compare_and_benchmark_are_agent_safe(self) -> None:
        comparison = agent_compare(
            {"workload": "european_call", "config": {"n_paths": 512, "n_steps": 8, "seed": 23}}
        )
        benchmark = agent_benchmark({"profile": "compact"})

        self.assertTrue(comparison["ok"])
        self.assertIn("alternatives", comparison)
        self.assertEqual(benchmark["status"], "dry_run")
        self.assertIn("cargo run -p mc-bench", benchmark["command"])

    def test_reproduce_accepts_agent_manifest(self) -> None:
        executed = agent_execute(
            {"workload": "down_and_out_call", "config": {"n_paths": 128, "n_steps": 4, "seed": 29}}
        )
        reproduced = agent_reproduce({"manifest": executed["manifest"]})

        self.assertTrue(reproduced["ok"])
        self.assertIn("price_down_and_out_call", reproduced["reproduction"]["python"])


if __name__ == "__main__":
    unittest.main()

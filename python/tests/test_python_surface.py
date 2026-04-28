import unittest

from mc_library import BenchmarkResult, MethodRecommendation, recommend_method


class PythonSurfaceTests(unittest.TestCase):
    def test_method_recommendation_defaults_to_fast_control_variate(self) -> None:
        recommendation = recommend_method(
            workload_family="european_call",
            n_paths=100_000,
            n_steps=64,
        )

        self.assertIsInstance(recommendation, MethodRecommendation)
        self.assertEqual(recommendation.method_id, "control_variates")
        self.assertEqual(recommendation.sampling, "pseudorandom")
        self.assertEqual(recommendation.technique, "control_variate")

    def test_method_recommendation_can_choose_sobol_bridge(self) -> None:
        recommendation = recommend_method(
            workload_family="down_and_out_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=True,
        )

        self.assertEqual(recommendation.method_id, "scrambled_sobol_brownian_bridge")
        self.assertEqual(recommendation.sampling, "scrambled_sobol_brownian_bridge")
        self.assertEqual(recommendation.technique, "control_variate")

    def test_method_recommendation_can_choose_mlqmc(self) -> None:
        recommendation = recommend_method(
            workload_family="arithmetic_asian_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=True,
        )

        self.assertEqual(recommendation.method_id, "multilevel_randomized_qmc")
        self.assertEqual(recommendation.sampling, "scrambled_sobol")
        self.assertEqual(recommendation.technique, "standard")

    def test_method_recommendation_can_choose_mlmc(self) -> None:
        recommendation = recommend_method(
            workload_family="arithmetic_asian_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=False,
        )

        self.assertEqual(recommendation.method_id, "multilevel_monte_carlo")
        self.assertEqual(recommendation.sampling, "pseudorandom")
        self.assertEqual(recommendation.technique, "standard")

    def test_benchmark_result_shape_is_public(self) -> None:
        result = BenchmarkResult(
            benchmark_name="example",
            backend="cpu_native",
            methodology="example_method",
            per_iteration_ms=1.25,
            metric_name="price_estimate",
            metric_value=9.4,
        )

        self.assertEqual(result.per_iteration_ms, 1.25)


if __name__ == "__main__":
    unittest.main()

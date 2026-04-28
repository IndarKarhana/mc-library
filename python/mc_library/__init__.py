"""Python-facing helpers for mc-library.

This package is intentionally thin for now: it preserves typed, inspectable
contracts while the compiled Python extension surface is still being designed.
"""

from .benchmarks import BenchmarkResult, run_benchmarks
from .methods import MethodRecommendation, recommend_method

__all__ = [
    "BenchmarkResult",
    "MethodRecommendation",
    "recommend_method",
    "run_benchmarks",
]

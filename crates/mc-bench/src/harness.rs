use std::collections::BTreeMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use mc_core::{
    plan_execution, BackendId, BackendPreference, BackendSupportReport, PlannerMode, RunConfig,
};
use mc_schema::{
    validate_simulation_spec, AxisKind, AxisSpec, Expr, ObservationSpec, ParameterSpec,
    RandomVarSpec, ReductionSpec, SimulationSpec, StateUpdate, StateVarSpec, StepSpec,
};

use crate::result::{BenchmarkReport, BenchmarkResult};

pub fn run_default_benchmarks() -> BenchmarkReport {
    let spec = sample_spec(false);

    BenchmarkReport {
        generated_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_millis(),
        results: vec![
            benchmark_schema_validation(&spec, 10_000),
            benchmark_planner_overhead(&spec, 10_000),
            benchmark_planner_choice_accuracy(),
        ],
    }
}

fn benchmark_schema_validation(spec: &SimulationSpec, iterations: usize) -> BenchmarkResult {
    let started = Instant::now();

    for _ in 0..iterations {
        let diagnostics = validate_simulation_spec(spec);
        if !diagnostics.is_empty() {
            panic!("expected no diagnostics in validation benchmark: {diagnostics:?}");
        }
    }

    let elapsed = started.elapsed();
    let total_runtime_ms = elapsed.as_secs_f64() * 1_000.0;
    let per_iteration_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    let throughput_per_sec = if elapsed.as_secs_f64() == 0.0 {
        iterations as f64
    } else {
        iterations as f64 / elapsed.as_secs_f64()
    };

    BenchmarkResult {
        benchmark_name: "schema_validation".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-schema::validate_simulation_spec".to_string(),
        backend: "cpu_native".to_string(),
        planner_mode: "n/a".to_string(),
        iterations,
        total_runtime_ms,
        per_iteration_us,
        throughput_per_sec,
        metric_name: None,
        metric_value: None,
    }
}

fn benchmark_planner_overhead(spec: &SimulationSpec, iterations: usize) -> BenchmarkResult {
    let support = vec![
        BackendSupportReport::supported(BackendId::CpuNative),
        BackendSupportReport::supported(BackendId::NvidiaCuda),
        BackendSupportReport::supported(BackendId::AppleMetal),
    ];

    let started = Instant::now();

    for _ in 0..iterations {
        let plan = plan_execution(
            spec,
            RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            &support,
        )
        .expect("planner benchmark should produce an execution plan");

        if plan.backend != BackendId::NvidiaCuda {
            panic!("expected planner to choose nvidia in benchmark scenario");
        }
    }

    let elapsed = started.elapsed();
    let total_runtime_ms = elapsed.as_secs_f64() * 1_000.0;
    let per_iteration_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    let throughput_per_sec = if elapsed.as_secs_f64() == 0.0 {
        iterations as f64
    } else {
        iterations as f64 / elapsed.as_secs_f64()
    };

    BenchmarkResult {
        benchmark_name: "planner_overhead_auto".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::plan_execution".to_string(),
        backend: "planner".to_string(),
        planner_mode: "balanced".to_string(),
        iterations,
        total_runtime_ms,
        per_iteration_us,
        throughput_per_sec,
        metric_name: None,
        metric_value: None,
    }
}

fn benchmark_planner_choice_accuracy() -> BenchmarkResult {
    #[derive(Clone)]
    struct Scenario {
        spec: SimulationSpec,
        run_config: RunConfig,
        support: Vec<BackendSupportReport>,
        expected: BackendId,
    }

    let scenarios = vec![
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 10_000,
                n_steps: 50,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::CpuNative,
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::NvidiaCuda,
        },
        Scenario {
            spec: sample_spec(true),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::CpuNative,
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::unsupported(BackendId::NvidiaCuda, "cuda unavailable"),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::AppleMetal,
        },
    ];

    let iterations = scenarios.len();
    let started = Instant::now();
    let mut correct = 0usize;

    for scenario in &scenarios {
        let plan = plan_execution(
            &scenario.spec,
            scenario.run_config.clone(),
            &scenario.support,
        )
        .expect("planner scenario should produce execution plan");
        if plan.backend == scenario.expected {
            correct += 1;
        }
    }

    let elapsed = started.elapsed();
    let total_runtime_ms = elapsed.as_secs_f64() * 1_000.0;
    let per_iteration_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    let throughput_per_sec = if elapsed.as_secs_f64() == 0.0 {
        iterations as f64
    } else {
        iterations as f64 / elapsed.as_secs_f64()
    };

    let accuracy_pct = (correct as f64 / iterations as f64) * 100.0;

    BenchmarkResult {
        benchmark_name: "planner_choice_accuracy".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::plan_execution".to_string(),
        backend: "planner".to_string(),
        planner_mode: "balanced".to_string(),
        iterations,
        total_runtime_ms,
        per_iteration_us,
        throughput_per_sec,
        metric_name: Some("accuracy_pct".to_string()),
        metric_value: Some(accuracy_pct),
    }
}

fn sample_spec(with_conditional: bool) -> SimulationSpec {
    let mut axes = BTreeMap::new();
    axes.insert(
        "path".to_string(),
        AxisSpec {
            name: "path".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: true,
            ordered: false,
        },
    );
    axes.insert(
        "step".to_string(),
        AxisSpec {
            name: "step".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: false,
            ordered: true,
        },
    );

    let update_expr = if with_conditional {
        Expr::BinaryOp {
            op: "gt".to_string(),
            lhs: Box::new(Expr::StateRef {
                value: "price".to_string(),
            }),
            rhs: Box::new(Expr::Literal { value: 0.0 }),
        }
    } else {
        Expr::StateRef {
            value: "price".to_string(),
        }
    };

    SimulationSpec {
        schema_version: "0.1".to_string(),
        name: "benchmark_case".to_string(),
        version: "0.1.0".to_string(),
        parameters: vec![ParameterSpec {
            name: "s0".to_string(),
            dtype: "float64".to_string(),
        }],
        axes,
        random_variables: vec![RandomVarSpec {
            name: "z".to_string(),
            distribution: "normal".to_string(),
            dtype: "float32".to_string(),
            axes: vec!["step".to_string()],
        }],
        state_variables: vec![StateVarSpec {
            name: "price".to_string(),
            dtype: "float32".to_string(),
            init: Expr::ParameterRef {
                value: "s0".to_string(),
            },
        }],
        steps: vec![StepSpec {
            name: "advance".to_string(),
            axis: "step".to_string(),
            updates: vec![StateUpdate {
                target: "price".to_string(),
                expr: update_expr,
            }],
        }],
        observations: vec![ObservationSpec {
            name: "payoff".to_string(),
            expr: Expr::StateRef {
                value: "price".to_string(),
            },
        }],
        reductions: vec![ReductionSpec {
            name: "expected_payoff".to_string(),
            op: "mean".to_string(),
            source: "payoff".to_string(),
            axes: vec!["path".to_string()],
        }],
    }
}

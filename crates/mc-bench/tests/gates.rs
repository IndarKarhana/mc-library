use mc_bench::run_default_benchmarks;

fn find_metric<'a>(
    name: &str,
    report: &'a mc_bench::BenchmarkReport,
) -> &'a mc_bench::BenchmarkResult {
    report
        .results
        .iter()
        .find(|r| r.benchmark_name == name)
        .unwrap_or_else(|| panic!("missing benchmark result '{name}'"))
}

#[test]
fn benchmark_gates_hold_for_current_internal_suite() {
    let report = run_default_benchmarks();

    let schema_validation = find_metric("schema_validation", &report);
    assert!(
        schema_validation.per_iteration_us < 50.0,
        "schema_validation gate failed: per_iteration_us={} expected<50",
        schema_validation.per_iteration_us
    );

    let planner_overhead = find_metric("planner_overhead_auto", &report);
    assert!(
        planner_overhead.per_iteration_us < 10.0,
        "planner_overhead_auto gate failed: per_iteration_us={} expected<10",
        planner_overhead.per_iteration_us
    );

    let planner_accuracy = find_metric("planner_choice_accuracy", &report);
    let accuracy = planner_accuracy
        .metric_value
        .expect("planner choice accuracy benchmark must contain metric_value");
    assert!(
        accuracy >= 75.0,
        "planner_choice_accuracy gate failed: accuracy_pct={} expected>=75",
        accuracy
    );

    let rust_mc = find_metric("mc_cpu_european_call_rust", &report);
    assert!(
        rust_mc.total_runtime_ms > 0.0,
        "mc_cpu_european_call_rust gate failed: expected benchmark presence and positive runtime"
    );
    assert_eq!(rust_mc.methodology.as_deref(), Some("stepwise_paths"));

    if let Some(numpy) = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_numpy")
    {
        assert!(
            rust_mc.per_iteration_us < numpy.per_iteration_us,
            "competitiveness gate failed: rust per_iteration_us={} numpy per_iteration_us={}",
            rust_mc.per_iteration_us,
            numpy.per_iteration_us
        );
    }

    if let Some(numba) = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_numba")
    {
        assert!(
            rust_mc.per_iteration_us < numba.per_iteration_us,
            "competitiveness gate failed: rust per_iteration_us={} numba per_iteration_us={}",
            rust_mc.per_iteration_us,
            numba.per_iteration_us
        );
    }

    let rust_terminal = find_metric("mc_cpu_european_call_rust_terminal", &report);
    assert!(
        rust_terminal.total_runtime_ms > 0.0,
        "mc_cpu_european_call_rust_terminal gate failed: expected benchmark presence and positive runtime"
    );
}

use mc_bench::run_default_benchmarks;

#[test]
fn benchmark_harness_produces_non_empty_results() {
    let report = run_default_benchmarks();
    assert!(
        !report.results.is_empty(),
        "expected benchmark results to be non-empty"
    );
}

#[test]
fn benchmark_metrics_are_non_negative() {
    let report = run_default_benchmarks();
    for result in &report.results {
        assert!(result.total_runtime_ms >= 0.0);
        assert!(result.per_iteration_us >= 0.0);
        assert!(result.throughput_per_sec >= 0.0);
    }
}

#[test]
fn planner_accuracy_benchmark_has_accuracy_metric() {
    let report = run_default_benchmarks();
    let planner_accuracy = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "planner_choice_accuracy")
        .expect("planner choice accuracy benchmark should be present");

    assert_eq!(
        planner_accuracy.metric_name.as_deref(),
        Some("accuracy_pct")
    );
    let accuracy = planner_accuracy
        .metric_value
        .expect("planner accuracy metric should contain a value");
    assert!(accuracy >= 0.0);
    assert!(accuracy <= 100.0);
}

use mc_core::{
    BackendDecisionReport, BackendExecutionInput, BackendId, CpuNativeBackend, EuropeanCallConfig,
    ExecutionPlan, FeatureSummary, PlannerMode, RejectedBackend, RuntimeBackend, SupportLevel,
};

fn test_plan() -> ExecutionPlan {
    ExecutionPlan {
        backend: BackendId::CpuNative,
        planner_mode: PlannerMode::Balanced,
        n_paths: 100_000,
        n_steps: 64,
        features: FeatureSummary::default(),
        decision_report: BackendDecisionReport {
            selected_backend: BackendId::CpuNative,
            reasons: vec!["unit-test".to_string()],
            rejected_backends: vec![RejectedBackend {
                backend: BackendId::NvidiaCuda,
                reason: "unit-test".to_string(),
            }],
        },
    }
}

#[test]
fn cpu_backend_discovers_host_device() {
    let backend = CpuNativeBackend::new();
    let devices = backend.discover_devices();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_id, "cpu:host");
}

#[test]
fn cpu_backend_reports_supported_for_cpu_plan() {
    let backend = CpuNativeBackend::new();
    let mut devices = backend.discover_devices();
    let device = devices.remove(0);
    let report = backend.supports(&test_plan(), &device);
    assert_eq!(report.support_level, SupportLevel::Supported);
}

#[test]
fn cpu_backend_execute_is_deterministic_for_same_seed() {
    let backend = CpuNativeBackend::new();
    let mut devices = backend.discover_devices();
    let device = devices.remove(0);
    let artifact = backend
        .compile(&test_plan(), &device)
        .expect("cpu backend compile should succeed");

    let cfg = EuropeanCallConfig {
        n_paths: 80_000,
        n_steps: 64,
        seed: 99,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };

    let run1 = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("cpu backend execute should succeed");
    let run2 = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("cpu backend execute should succeed");

    assert_eq!(run1.price, run2.price);
    assert_eq!(run1.stderr, run2.stderr);
}

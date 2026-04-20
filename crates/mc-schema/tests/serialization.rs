use std::collections::BTreeMap;

use mc_schema::{
    check_schema_compatibility, AxisKind, AxisSpec, Expr, ObservationSpec, ParameterSpec,
    RandomVarSpec, ReductionSpec, SimulationSpec, StateUpdate, StateVarSpec, StepSpec,
};

fn sample_spec() -> SimulationSpec {
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

    SimulationSpec {
        schema_version: "0.1".to_string(),
        name: "serialization_case".to_string(),
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
            axes: vec![],
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
            axis: "path".to_string(),
            updates: vec![StateUpdate {
                target: "price".to_string(),
                expr: Expr::StateRef {
                    value: "price".to_string(),
                },
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

#[test]
fn simulation_spec_json_roundtrip_is_lossless() {
    let spec = sample_spec();
    let json = serde_json::to_string(&spec).expect("serialize should succeed");
    let decoded: SimulationSpec = serde_json::from_str(&json).expect("deserialize should succeed");
    assert_eq!(decoded, spec);
}

#[test]
fn compatibility_accepts_current_schema_version() {
    let report = check_schema_compatibility("0.1");
    assert!(
        report.supported,
        "expected compatibility report to be supported"
    );
}

#[test]
fn compatibility_rejects_newer_schema_minor() {
    let report = check_schema_compatibility("0.2");
    assert!(
        !report.supported,
        "expected compatibility report to reject newer schema"
    );
}

#[test]
fn compatibility_rejects_invalid_version_format() {
    let report = check_schema_compatibility("v0.1");
    assert!(
        !report.supported,
        "expected invalid version format to be unsupported"
    );
}

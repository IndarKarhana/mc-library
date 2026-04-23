use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    pub benchmark_name: String,
    pub benchmark_version: String,
    pub implementation: String,
    pub backend: String,
    pub methodology: Option<String>,
    pub planner_mode: String,
    pub iterations: usize,
    pub total_runtime_ms: f64,
    pub per_iteration_us: f64,
    pub throughput_per_sec: f64,
    pub metric_name: Option<String>,
    pub metric_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkReport {
    pub generated_at_unix_ms: u128,
    pub results: Vec<BenchmarkResult>,
}

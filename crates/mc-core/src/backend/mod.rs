use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    european_call_price_mc_cpu, BackendId, EuropeanCallConfig, ExecutionPlan, PlannerMode,
    SupportLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendInfo {
    pub backend_id: BackendId,
    pub display_name: String,
    pub version: String,
    pub platform: String,
    pub supported_precisions: Vec<String>,
    pub supported_rngs: Vec<String>,
    pub supported_sampling_modes: Vec<String>,
    pub supported_reduction_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub device_id: String,
    pub backend_id: BackendId,
    pub name: String,
    pub vendor: String,
    pub supports_float64: bool,
    pub supports_unified_memory: bool,
    pub max_threads_hint: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportReport {
    pub backend_id: BackendId,
    pub device_id: String,
    pub support_level: SupportLevel,
    pub unsupported_features: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostEstimate {
    pub backend_id: BackendId,
    pub device_id: String,
    pub estimated_compile_ms: f64,
    pub estimated_runtime_ms: f64,
    pub estimated_total_ms: f64,
    pub estimated_peak_memory_mb: f64,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReproSupport {
    pub supports_same_backend_exact: bool,
    pub supports_same_backend_deterministic: bool,
    pub supports_cross_backend_statistical: bool,
    pub supports_stable_chunking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledArtifact {
    pub artifact_id: String,
    pub backend_id: BackendId,
    pub device_id: String,
    pub n_paths: usize,
    pub n_steps: usize,
    pub planner_mode: PlannerMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunOutput {
    pub price: f64,
    pub stderr: f64,
    pub runtime_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendExecutionInput {
    EuropeanCall(EuropeanCallConfig),
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("device '{0}' is not available for this backend")]
    UnknownDevice(String),
    #[error("execution input is not compatible with compiled artifact")]
    IncompatibleExecutionInput,
}

pub trait RuntimeBackend {
    fn backend_id(&self) -> BackendId;
    fn describe_backend(&self) -> BackendInfo;
    fn discover_devices(&self) -> Vec<DeviceInfo>;
    fn supports(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport;
    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate;
    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError>;
    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError>;
    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport;
}

#[derive(Debug, Clone, Default)]
pub struct CpuNativeBackend;

impl CpuNativeBackend {
    pub fn new() -> Self {
        Self
    }

    fn validate_device(&self, device: &DeviceInfo) -> Result<(), BackendError> {
        if device.backend_id != BackendId::CpuNative || device.device_id != "cpu:host" {
            return Err(BackendError::UnknownDevice(device.device_id.clone()));
        }
        Ok(())
    }
}

impl RuntimeBackend for CpuNativeBackend {
    fn backend_id(&self) -> BackendId {
        BackendId::CpuNative
    }

    fn describe_backend(&self) -> BackendInfo {
        BackendInfo {
            backend_id: BackendId::CpuNative,
            display_name: "CPU Native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: "cpu".to_string(),
            supported_precisions: vec!["float32".to_string(), "float64".to_string()],
            supported_rngs: vec!["xorshift64*".to_string()],
            supported_sampling_modes: vec!["iid".to_string()],
            supported_reduction_ops: vec![
                "sum".to_string(),
                "mean".to_string(),
                "variance".to_string(),
                "std".to_string(),
                "min".to_string(),
                "max".to_string(),
            ],
        }
    }

    fn discover_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            device_id: "cpu:host".to_string(),
            backend_id: BackendId::CpuNative,
            name: "Host CPU".to_string(),
            vendor: "generic".to_string(),
            supports_float64: true,
            supports_unified_memory: true,
            max_threads_hint: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        }]
    }

    fn supports(&self, _plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::CpuNative,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        SupportReport {
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::Supported,
            unsupported_features: vec![],
            warnings: vec![],
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 5_000_000.0).max(0.01);

        CostEstimate {
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            estimated_compile_ms: 0.0,
            estimated_runtime_ms,
            estimated_total_ms: estimated_runtime_ms,
            estimated_peak_memory_mb: 8.0,
            confidence: "low".to_string(),
        }
    }

    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;

        Ok(CompiledArtifact {
            artifact_id: format!(
                "cpu-native:{}:{}:{}",
                plan.n_paths, plan.n_steps, plan.features.step_count
            ),
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            n_paths: plan.n_paths,
            n_steps: plan.n_steps,
            planner_mode: plan.planner_mode,
        })
    }

    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError> {
        if artifact.backend_id != BackendId::CpuNative {
            return Err(BackendError::IncompatibleExecutionInput);
        }

        let started = Instant::now();

        let result = match input {
            BackendExecutionInput::EuropeanCall(cfg) => european_call_price_mc_cpu(cfg),
        };

        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        Ok(RunOutput {
            price: result.price,
            stderr: result.stderr,
            runtime_ms,
        })
    }

    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport {
        ReproSupport {
            supports_same_backend_exact: true,
            supports_same_backend_deterministic: true,
            supports_cross_backend_statistical: true,
            supports_stable_chunking: true,
        }
    }
}

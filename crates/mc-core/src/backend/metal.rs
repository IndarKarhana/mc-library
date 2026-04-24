use std::{fs, path::PathBuf, process::Command, thread, time::Instant};

use super::{
    compile_gpu_fallback_artifact, execute_gpu_fallback, make_native_artifact_metadata,
    plan_gpu_chunking, BackendError, BackendExecutionInput, BackendId, BackendInfo,
    CompiledArtifact, CostEstimate, DeviceInfo, ExecutionPlan, GpuBufferBinding,
    GpuBufferDirection, GpuChunkingConfig, GpuKernelContract, GpuLaunchDimensions,
    GpuScalarBinding, GpuValueType, ReproSupport, RuntimeBackend, SupportReport,
};
use crate::{
    runtime::cpu::generate_stepwise_standard_normals_f32, MonteCarloTechnique, SupportLevel,
};

pub fn metal_native_feature_enabled() -> bool {
    cfg!(feature = "metal-native")
}

const FIRST_METAL_KERNEL_ENTRY_POINT: &str = "mc_metal_european_call_stepwise_v1";
const FIRST_METAL_KERNEL_FAMILY: &str = "european_call_stepwise_v1";
const FIRST_METAL_KERNEL_SOURCE_MODULE: &str =
    "crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal";
const FIRST_METAL_KERNEL_SOURCE: &str = include_str!("kernels/european_call_stepwise_v1.metal");
const FIRST_METAL_SWIFT_RUNNER_SOURCE: &str = include_str!("tools/run_metal_stepwise.swift");

#[derive(Debug, Clone, Default)]
pub struct AppleMetalBackend;

impl AppleMetalBackend {
    pub fn new() -> Self {
        Self
    }

    fn validate_device(&self, device: &DeviceInfo) -> Result<(), BackendError> {
        if device.backend_id != BackendId::AppleMetal || !device.device_id.starts_with("metal:") {
            return Err(BackendError::UnknownDevice(device.device_id.clone()));
        }
        Ok(())
    }
}

impl RuntimeBackend for AppleMetalBackend {
    fn backend_id(&self) -> BackendId {
        BackendId::AppleMetal
    }

    fn describe_backend(&self) -> BackendInfo {
        BackendInfo {
            backend_id: BackendId::AppleMetal,
            display_name: "Apple Metal".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: "metal".to_string(),
            supported_precisions: vec!["float32".to_string()],
            supported_rngs: vec!["philox".to_string(), "sobol".to_string()],
            supported_sampling_modes: vec!["iid".to_string(), "qmc".to_string()],
            supported_reduction_ops: vec![
                "sum".to_string(),
                "mean".to_string(),
                "variance".to_string(),
                "min".to_string(),
                "max".to_string(),
            ],
        }
    }

    fn discover_devices(&self) -> Vec<DeviceInfo> {
        discover_apple_metal_devices()
    }

    fn supports(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::AppleMetal,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        let mut warnings = vec![
            "Apple Metal backend currently executes through delegated CPU fallback while native kernels are in progress"
                .to_string(),
        ];
        let mut unsupported_features = vec!["native_metal_execution_not_implemented".to_string()];

        if !supports_first_metal_kernel_shape(plan) {
            unsupported_features.push("first_metal_kernel_shape_not_supported".to_string());
            warnings.push(
                "first staged Metal kernel currently targets the narrow European-call stepwise workload"
                    .to_string(),
            );
        }

        if metal_native_feature_enabled() {
            if probe_metal_toolchain() {
                warnings.push(
                    "metal-native feature enabled and Metal toolchain detected; host-side shader staging is active"
                        .to_string(),
                );
            } else {
                warnings.push(
                    "metal-native feature enabled but Metal toolchain was not detected on this machine"
                        .to_string(),
                );
            }
        } else {
            warnings.push(
                "enable the `metal-native` feature to validate host-side Metal staging in CI or locally"
                    .to_string(),
            );
        }

        SupportReport {
            backend_id: BackendId::AppleMetal,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::SupportedWithFallbacks,
            unsupported_features,
            warnings,
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 35_000_000.0).max(0.01);
        let estimated_compile_ms = if metal_native_feature_enabled() {
            1.0
        } else {
            1.5
        };
        let chunking = plan_gpu_chunking(
            plan.n_paths,
            device.memory_total_mb,
            GpuChunkingConfig {
                bytes_per_path: super::estimate_gpu_bytes_per_path(plan),
                target_utilization: 0.70,
                minimum_paths_per_chunk: 32_768,
                fallback_budget_mb: 6_144,
            },
        );

        CostEstimate {
            backend_id: BackendId::AppleMetal,
            device_id: device.device_id.clone(),
            estimated_compile_ms,
            estimated_runtime_ms,
            estimated_total_ms: estimated_compile_ms + estimated_runtime_ms,
            estimated_peak_memory_mb: chunking.estimated_peak_memory_mb as f64,
            confidence: if metal_native_feature_enabled() {
                "medium".to_string()
            } else {
                "low".to_string()
            },
        }
    }

    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;
        let compile_status = stage_native_metal_kernel(plan);
        let notes = metal_kernel_notes(plan, &compile_status);

        let native_artifact = Some(make_native_artifact_metadata(
            FIRST_METAL_KERNEL_FAMILY,
            FIRST_METAL_KERNEL_ENTRY_POINT,
            FIRST_METAL_KERNEL_SOURCE_MODULE,
            "metal_shading_language",
            "metal-native",
            compile_status.toolchain_available,
            compile_status.compile_requested,
            compile_status.compile_succeeded,
            compile_status.compiled_module_path,
            Some(first_metal_kernel_contract(plan)),
            notes,
        ));

        Ok(compile_gpu_fallback_artifact(
            BackendId::AppleMetal,
            "metal",
            plan,
            device,
            native_artifact,
        ))
    }

    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<super::RunOutput, BackendError> {
        if let Ok(run_output) = execute_native_metal_if_possible(artifact, input) {
            return Ok(run_output);
        }

        execute_gpu_fallback(BackendId::AppleMetal, artifact, input)
    }

    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport {
        ReproSupport {
            supports_same_backend_exact: false,
            supports_same_backend_deterministic: true,
            supports_cross_backend_statistical: true,
            supports_stable_chunking: true,
        }
    }
}

pub(crate) fn discover_apple_metal_devices() -> Vec<DeviceInfo> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }

    vec![DeviceInfo {
        device_id: "metal:0".to_string(),
        backend_id: BackendId::AppleMetal,
        name: "Apple GPU".to_string(),
        vendor: "apple".to_string(),
        memory_total_mb: None,
        memory_free_mb: None,
        supports_float64: false,
        supports_unified_memory: true,
        max_threads_hint: thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    }]
}

fn supports_first_metal_kernel_shape(plan: &ExecutionPlan) -> bool {
    plan.features.conditional_expression_count == 0 && plan.features.reduction_count <= 1
}

fn first_metal_kernel_contract(plan: &ExecutionPlan) -> GpuKernelContract {
    GpuKernelContract {
        kernel_family: FIRST_METAL_KERNEL_FAMILY.to_string(),
        entry_point: FIRST_METAL_KERNEL_ENTRY_POINT.to_string(),
        buffers: vec![
            GpuBufferBinding {
                binding_index: 0,
                name: "normals".to_string(),
                direction: GpuBufferDirection::Input,
                value_type: GpuValueType::Float32,
                element_count: plan.n_paths.saturating_mul(plan.n_steps),
            },
            GpuBufferBinding {
                binding_index: 1,
                name: "payoffs".to_string(),
                direction: GpuBufferDirection::Output,
                value_type: GpuValueType::Float32,
                element_count: plan.n_paths,
            },
        ],
        scalars: vec![
            GpuScalarBinding {
                binding_index: 2,
                name: "n_paths".to_string(),
                value_type: GpuValueType::Int32,
            },
            GpuScalarBinding {
                binding_index: 3,
                name: "n_steps".to_string(),
                value_type: GpuValueType::Int32,
            },
            GpuScalarBinding {
                binding_index: 4,
                name: "log_s0".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 5,
                name: "strike".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 6,
                name: "drift_dt".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 7,
                name: "vol_dt".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 8,
                name: "discount".to_string(),
                value_type: GpuValueType::Float32,
            },
        ],
        launch: GpuLaunchDimensions {
            logical_threads: plan.n_paths,
            threads_per_group_x: 256,
            threadgroups_x: (plan.n_paths as u32).div_ceil(256),
        },
    }
}

#[derive(Debug, Clone)]
struct MetalKernelStageStatus {
    toolchain_available: bool,
    compile_requested: bool,
    compile_succeeded: bool,
    compiled_module_path: Option<String>,
    diagnostics: Vec<String>,
}

fn metal_kernel_notes(
    plan: &ExecutionPlan,
    compile_status: &MetalKernelStageStatus,
) -> Vec<String> {
    let mut notes = vec![
        "host-side Metal shader ABI is staged but runtime still executes through delegated CPU fallback"
            .to_string(),
        format!(
            "validated target shape: n_paths={} n_steps={} conditional_expressions={}",
            plan.n_paths, plan.n_steps, plan.features.conditional_expression_count
        ),
        format!("kernel_entry_point={FIRST_METAL_KERNEL_ENTRY_POINT}"),
    ];

    if metal_native_feature_enabled() {
        notes.push(
            "feature gate enabled; native launch plumbing can be validated at compile time"
                .to_string(),
        );
    } else {
        notes.push(
            "feature gate disabled; artifact remains a native-ready manifest only".to_string(),
        );
    }

    notes.extend(compile_status.diagnostics.iter().cloned());

    notes
}

fn probe_metal_toolchain() -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }

    Command::new("xcrun")
        .args(["-sdk", "macosx", "metal", "-v"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn stage_native_metal_kernel(plan: &ExecutionPlan) -> MetalKernelStageStatus {
    let toolchain_available = probe_metal_toolchain();
    let compile_requested = metal_native_feature_enabled();

    if !compile_requested {
        return MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![
                "native Metal compile skipped because the `metal-native` feature is disabled"
                    .to_string(),
            ],
        };
    }

    if !toolchain_available {
        return MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![
                "native Metal compile requested but the Metal developer tools were not available on this machine"
                    .to_string(),
            ],
        };
    }

    match compile_metal_kernel_to_metallib(plan) {
        Ok(metallib_path) => MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: true,
            compiled_module_path: Some(metallib_path.display().to_string()),
            diagnostics: vec![format!(
                "native Metal library compilation succeeded for staged kernel: {}",
                metallib_path.display()
            )],
        },
        Err(error) => MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![format!("native Metal library compilation failed: {error}")],
        },
    }
}

fn execute_native_metal_if_possible(
    artifact: &CompiledArtifact,
    input: &BackendExecutionInput,
) -> Result<super::RunOutput, BackendError> {
    if artifact.backend_id != BackendId::AppleMetal {
        return Err(BackendError::IncompatibleExecutionInput);
    }

    if !metal_native_feature_enabled() || !cfg!(target_os = "macos") {
        return Err(BackendError::UnsupportedFeature(
            "native Metal execution requires the metal-native feature on macOS".to_string(),
        ));
    }

    let cfg = match input {
        BackendExecutionInput::EuropeanCall(cfg) => cfg,
    };

    if cfg.n_paths != artifact.n_paths || cfg.n_steps != artifact.n_steps {
        return Err(BackendError::IncompatibleExecutionInput);
    }

    if cfg.technique != MonteCarloTechnique::Standard {
        return Err(BackendError::UnsupportedFeature(
            "native Metal execution currently supports standard stepwise European call only"
                .to_string(),
        ));
    }

    let started = Instant::now();
    let normals = generate_stepwise_standard_normals_f32(cfg.seed, cfg.n_paths, cfg.n_steps);
    let result = execute_metal_stepwise_kernel(cfg, &normals)?;

    Ok(super::RunOutput {
        price: result.price,
        stderr: result.stderr,
        runtime_ms: started.elapsed().as_secs_f64() * 1_000.0,
    })
}

fn execute_metal_stepwise_kernel(
    cfg: &crate::EuropeanCallConfig,
    normals: &[f32],
) -> Result<crate::EuropeanCallResult, BackendError> {
    let output_dir = staged_metal_output_dir();
    fs::create_dir_all(&output_dir).map_err(|error| {
        BackendError::UnsupportedFeature(format!(
            "unable to create Metal runtime staging directory: {error}"
        ))
    })?;

    let source_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_runtime_{}paths_{}steps.metal",
        cfg.n_paths, cfg.n_steps
    ));
    let script_path = output_dir.join("run_metal_stepwise.swift");
    let normals_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_runtime_{}paths_{}steps_normals.bin",
        cfg.n_paths, cfg.n_steps
    ));

    fs::write(&source_path, FIRST_METAL_KERNEL_SOURCE).map_err(|error| {
        BackendError::UnsupportedFeature(format!("unable to write staged Metal source: {error}"))
    })?;
    fs::write(&script_path, FIRST_METAL_SWIFT_RUNNER_SOURCE).map_err(|error| {
        BackendError::UnsupportedFeature(format!(
            "unable to write staged Metal Swift runner: {error}"
        ))
    })?;

    let mut normals_bytes = Vec::with_capacity(std::mem::size_of_val(normals));
    for value in normals {
        normals_bytes.extend_from_slice(&value.to_ne_bytes());
    }
    fs::write(&normals_path, normals_bytes).map_err(|error| {
        BackendError::UnsupportedFeature(format!(
            "unable to write staged Metal normals buffer: {error}"
        ))
    })?;

    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;

    let output = Command::new("swift")
        .arg(script_path.to_string_lossy().as_ref())
        .arg(source_path.to_string_lossy().as_ref())
        .arg(normals_path.to_string_lossy().as_ref())
        .arg(cfg.n_paths.to_string())
        .arg(cfg.n_steps.to_string())
        .arg((cfg.s0.ln() as f32).to_string())
        .arg((cfg.k as f32).to_string())
        .arg(drift_dt.to_string())
        .arg(vol_dt.to_string())
        .arg(discount.to_string())
        .output()
        .map_err(|error| {
            BackendError::UnsupportedFeature(format!(
                "unable to spawn Swift Metal runtime helper: {error}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BackendError::UnsupportedFeature(format!(
            "Metal runtime helper failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload = stdout.trim();
    let mut parts = payload.split(',');
    let price = parts
        .next()
        .ok_or_else(|| {
            BackendError::UnsupportedFeature(
                "Metal runtime helper returned malformed price output".to_string(),
            )
        })?
        .parse::<f64>()
        .map_err(|error| {
            BackendError::UnsupportedFeature(format!(
                "unable to parse Metal runtime price output: {error}"
            ))
        })?;
    let stderr = parts
        .next()
        .ok_or_else(|| {
            BackendError::UnsupportedFeature(
                "Metal runtime helper returned malformed stderr output".to_string(),
            )
        })?
        .parse::<f64>()
        .map_err(|error| {
            BackendError::UnsupportedFeature(format!(
                "unable to parse Metal runtime stderr output: {error}"
            ))
        })?;

    Ok(crate::EuropeanCallResult { price, stderr })
}

#[cfg(all(test, feature = "metal-native", target_os = "macos"))]
mod tests {
    use super::*;
    use crate::{
        runtime::cpu::european_call_price_mc_stepwise_from_f32_normals, EuropeanCallConfig,
    };

    #[test]
    fn native_metal_stepwise_kernel_matches_cpu_reference_from_same_normals() {
        let cfg = EuropeanCallConfig {
            n_paths: 4_096,
            n_steps: 32,
            seed: 404,
            n_threads: 1,
            technique: MonteCarloTechnique::Standard,
            ..EuropeanCallConfig::default()
        };
        let normals = generate_stepwise_standard_normals_f32(cfg.seed, cfg.n_paths, cfg.n_steps);

        let metal = execute_metal_stepwise_kernel(&cfg, &normals)
            .expect("native Metal stepwise kernel should execute successfully");
        let cpu = european_call_price_mc_stepwise_from_f32_normals(&cfg, &normals);

        let price_error = (metal.price - cpu.price).abs();
        let stderr_error = (metal.stderr - cpu.stderr).abs();
        assert!(
            price_error <= 1e-3,
            "price mismatch too large: {price_error}"
        );
        assert!(
            stderr_error <= 1e-4,
            "stderr mismatch too large: {stderr_error}"
        );
    }
}

fn compile_metal_kernel_to_metallib(plan: &ExecutionPlan) -> Result<PathBuf, String> {
    let output_dir = staged_metal_output_dir();
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("unable to create Metal staging directory: {error}"))?;

    let source_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.metal",
        plan.n_paths, plan.n_steps
    ));
    let air_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.air",
        plan.n_paths, plan.n_steps
    ));
    let metallib_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.metallib",
        plan.n_paths, plan.n_steps
    ));

    fs::write(&source_path, FIRST_METAL_KERNEL_SOURCE)
        .map_err(|error| format!("unable to write staged Metal source: {error}"))?;

    let metal_output = Command::new("xcrun")
        .args([
            "-sdk",
            "macosx",
            "metal",
            source_path.to_string_lossy().as_ref(),
            "-o",
            air_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|error| format!("failed to spawn metal compiler: {error}"))?;

    if !metal_output.status.success() {
        let stderr = String::from_utf8_lossy(&metal_output.stderr);
        return Err(stderr.trim().to_string());
    }

    let metallib_output = Command::new("xcrun")
        .args([
            "-sdk",
            "macosx",
            "metallib",
            air_path.to_string_lossy().as_ref(),
            "-o",
            metallib_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|error| format!("failed to spawn metallib: {error}"))?;

    if !metallib_output.status.success() {
        let stderr = String::from_utf8_lossy(&metallib_output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(metallib_path)
}

fn staged_metal_output_dir() -> PathBuf {
    std::env::temp_dir().join("mc-library").join("metal")
}

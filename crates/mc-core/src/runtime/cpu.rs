use std::f64::consts::PI;
use std::thread;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EuropeanCallMethod {
    Auto,
    TerminalDistribution,
    StepwisePaths,
}

impl Default for EuropeanCallMethod {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonteCarloTechnique {
    Standard,
    Antithetic,
}

impl Default for MonteCarloTechnique {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
}

impl Default for EuropeanCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallResult {
    pub price: f64,
    pub stderr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallPricer {
    config: EuropeanCallConfig,
    method: EuropeanCallMethod,
}

impl Default for EuropeanCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl EuropeanCallPricer {
    pub fn new() -> Self {
        Self {
            config: EuropeanCallConfig::default(),
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn from_config(config: EuropeanCallConfig) -> Self {
        Self {
            config,
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn steps(mut self, value: usize) -> Self {
        self.config.n_steps = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn method(mut self, value: EuropeanCallMethod) -> Self {
        self.method = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.method = EuropeanCallMethod::TerminalDistribution;
        self
    }

    pub fn stepwise(mut self) -> Self {
        self.method = EuropeanCallMethod::StepwisePaths;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn config(&self) -> &EuropeanCallConfig {
        &self.config
    }

    pub fn methodology(&self) -> EuropeanCallMethod {
        self.method
    }

    pub fn price(&self) -> EuropeanCallResult {
        european_call_price_mc_cpu_with_method(&self.config, self.method)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloRng {
    state: u64,
    cached_normal: Option<f64>,
}

impl MonteCarloRng {
    pub fn new(seed: u64) -> Self {
        let non_zero_seed = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };

        Self {
            state: non_zero_seed,
            cached_normal: None,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64* for a small deterministic PRNG with low overhead.
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_f64_open01(&mut self) -> f64 {
        // Use top 53 bits to produce a uniform in (0, 1).
        let raw = self.next_u64() >> 11;
        let value = (raw as f64) * (1.0 / ((1u64 << 53) as f64));
        value.max(f64::MIN_POSITIVE)
    }

    pub fn standard_normal(&mut self) -> f64 {
        if let Some(cached) = self.cached_normal.take() {
            return cached;
        }

        // Box-Muller transform. Cache one sample to halve transcendental calls.
        let u1 = self.next_f64_open01();
        let u2 = self.next_f64_open01();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;
        let z0 = radius * theta.cos();
        let z1 = radius * theta.sin();
        self.cached_normal = Some(z1);
        z0
    }
}

pub fn european_call_price_mc_cpu(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    european_call_price_mc_cpu_with_method(cfg, EuropeanCallMethod::Auto)
}

pub fn european_call_price_mc_cpu_with_method(
    cfg: &EuropeanCallConfig,
    method: EuropeanCallMethod,
) -> EuropeanCallResult {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");

    match method {
        EuropeanCallMethod::Auto | EuropeanCallMethod::TerminalDistribution => {
            european_call_price_mc_cpu_terminal(cfg)
        }
        EuropeanCallMethod::StepwisePaths => european_call_price_mc_cpu_stepwise(cfg),
    }
}

pub fn european_call_price_mc_cpu_terminal(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    if cfg.technique == MonteCarloTechnique::Antithetic {
        return simulate_terminal_antithetic(cfg);
    }

    // For European calls under GBM, we can sample terminal distribution directly:
    // S_T = S_0 * exp((r - 0.5*sigma^2)T + sigma*sqrt(T)*Z)
    // This is equivalent in distribution to step-by-step simulation and is much faster.
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_terminal_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            vol_t,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_terminal_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

pub fn european_call_price_mc_cpu_stepwise(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    if cfg.technique == MonteCarloTechnique::Antithetic {
        return simulate_stepwise_antithetic(cfg);
    }

    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_stepwise_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_terminal_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                vol_t,
                discount,
                technique,
            )
        }));
    }

    // Join in spawn order so reduction order is deterministic across runs.
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_stepwise_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_stepwise_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
                technique,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let z = rng.standard_normal();
                let payoff_a = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                let payoff_b = european_call_payoff_from_shock(s0, k, drift_t, vol_t, -z, discount);
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_stepwise_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let mut log_a = s0.ln();
                let mut log_b = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                }

                let payoff_a = (log_a.exp() - k).max(0.0) * discount;
                let payoff_b = (log_b.exp() - k).max(0.0) * discount;
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn summarize_payoffs(n_paths: usize, payoff_sum: f64, payoff_sq_sum: f64) -> EuropeanCallResult {
    let n = n_paths as f64;
    let price = payoff_sum / n;
    let variance = (payoff_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

fn summarize_block_estimates(
    block_count: usize,
    block_sum: f64,
    block_sq_sum: f64,
) -> EuropeanCallResult {
    let n = block_count as f64;
    let price = block_sum / n;
    let variance = (block_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

fn european_call_payoff_from_shock(
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    z: f64,
    discount: f64,
) -> f64 {
    let s_t = s0 * (drift_t + vol_t * z).exp();
    (s_t - k).max(0.0) * discount
}

fn simulate_terminal_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let z = rng.standard_normal();
        let payoff_a = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, z, discount);
        let payoff_b = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, -z, discount);
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_stepwise_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = cfg.s0.ln();
        let mut log_b = cfg.s0.ln();
        for _ in 0..cfg.n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
        }

        let payoff_a = (log_a.exp() - cfg.k).max(0.0) * discount;
        let payoff_b = (log_b.exp() - cfg.k).max(0.0) * discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn resolved_thread_count(requested_threads: usize) -> usize {
    if requested_threads > 0 {
        return requested_threads;
    }

    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn derive_chunk_seed(base_seed: u64, chunk_index: u64) -> u64 {
    splitmix64(base_seed.wrapping_add(chunk_index.wrapping_mul(0x9E37_79B9_7F4A_7C15)))
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

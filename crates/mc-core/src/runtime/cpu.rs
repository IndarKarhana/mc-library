use std::f64::consts::PI;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EuropeanCallResult {
    pub price: f64,
    pub stderr: f64,
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
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");

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
        )
    } else {
        simulate_terminal_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    let n = cfg.n_paths as f64;
    let price = payoff_sum / n;
    let variance = (payoff_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
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
        handles.push(thread::spawn(move || {
            simulate_terminal_chunk(seed, n_paths_chunk, s0, k, drift_t, vol_t, discount)
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

fn simulate_terminal_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    for _ in 0..n_paths {
        let z = rng.standard_normal();
        let s_t = s0 * (drift_t + vol_t * z).exp();
        let payoff = (s_t - k).max(0.0) * discount;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    (payoff_sum, payoff_sq_sum)
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

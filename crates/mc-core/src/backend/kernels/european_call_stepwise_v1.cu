extern "C" __global__ void mc_cuda_european_call_stepwise_v1(
    const double* normals,
    double* payoffs,
    int n_paths,
    int n_steps,
    double log_s0,
    double strike,
    double drift_dt,
    double vol_dt,
    double discount
) {
    int path_idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (path_idx >= n_paths) {
        return;
    }

    double log_s_t = log_s0;
    int base_offset = path_idx * n_steps;

    for (int step = 0; step < n_steps; ++step) {
        double z = normals[base_offset + step];
        log_s_t += drift_dt + vol_dt * z;
    }

    double s_t = exp(log_s_t);
    double payoff = s_t > strike ? (s_t - strike) * discount : 0.0;
    payoffs[path_idx] = payoff;
}

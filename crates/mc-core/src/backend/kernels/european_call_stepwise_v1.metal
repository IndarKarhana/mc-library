#include <metal_stdlib>
using namespace metal;

kernel void mc_metal_european_call_stepwise_v1(
    device const float* normals [[buffer(0)]],
    device float* payoffs [[buffer(1)]],
    constant int& n_paths [[buffer(2)]],
    constant int& n_steps [[buffer(3)]],
    constant float& log_s0 [[buffer(4)]],
    constant float& strike [[buffer(5)]],
    constant float& drift_dt [[buffer(6)]],
    constant float& vol_dt [[buffer(7)]],
    constant float& discount [[buffer(8)]],
    uint gid [[thread_position_in_grid]]
) {
    if (gid >= static_cast<uint>(n_paths)) {
        return;
    }

    float log_s_t = log_s0;
    uint base_offset = gid * static_cast<uint>(n_steps);

    for (int step = 0; step < n_steps; ++step) {
        float z = normals[base_offset + static_cast<uint>(step)];
        log_s_t += drift_dt + vol_dt * z;
    }

    float s_t = exp(log_s_t);
    float payoff = s_t > strike ? (s_t - strike) * discount : 0.0f;
    payoffs[gid] = payoff;
}

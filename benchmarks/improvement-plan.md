# Competitiveness Plan

Current tracked leaders:
- Rust fair CPU baseline (`mc_cpu_european_call_rust`, step-wise): `13.269 ms`
- Native Metal GBM baseline (`mc_metal_european_call_native`): `1.684 ms`
- Down-and-out breadth check: CPU `19.072 ms`, Metal `0.693 ms`
- Measured planner choice accuracy: `87.5%`

Status: Rust currently leads the available CPU baselines for the tracked fair European workload.

Maintain lead plan:
- Keep the step-wise benchmark as the primary competitive claim.
- Keep RNG and loop hot path allocation-free.
- Keep breadth claims tied to the workloads we have actually benchmarked: European, arithmetic Asian, down-and-out, basket, and Gaussian UQ.
- Expand competitor matrix to GPU baselines (JAX/CuPy/PyTorch/CUDA-native) when hardware is available.
- First randomized-QMC pricing surface is live via randomized Halton (`78.311 ms`), but it is currently a quality-first pricing path rather than a speed leader.
- Latin hypercube pricing is live (`65.046 ms`) as the first non-QMC structured-sampling breadth path.
- Scrambled Sobol pricing is live (`80.240 ms`) as the stronger QMC breadth path.
- Scrambled Sobol with Brownian bridge pricing is live (`104.042 ms`) for path construction experiments.
- QMC generation scoreboard is live: Rust scrambled Sobol generation `74.359 ms`, SciPy scrambled Sobol generation `120.144 ms` (`1.62x` Rust/SciPy speedup).
- Arithmetic Asian MLMC is live (`4.528 ms`) with adaptive tolerance planning as the first multilevel CPU reference path.
- Arithmetic Asian MLQMC is live (`5.755 ms`) with replicated scrambling and adaptive tolerance planning.
- Gaussian UQ benchmark is live: Latin hypercube `2.103 ms`, abs error `0.000039` vs pseudorandom abs error `0.006344`.
- Basket-call QMC breadth is live: scrambled Sobol basket pricing `7.167 ms`, Latin-hypercube stderr ratio vs pseudorandom `0.997`.
- Preserve the randomized-QMC quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing sequence generation and path construction.
- Preserve the Latin-hypercube quality gain (`stderr_ratio_vs_standard = 0.410`) while benchmarking it across more workload families.
- Preserve the Sobol Brownian-bridge quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing its current runtime overhead.
- Track arithmetic Asian MLMC quality (`stderr_ratio_vs_standard = 2.013`) and calibrate tolerance defaults before claiming it as a default winner.
- Preserve arithmetic Asian replicated MLQMC quality (`stderr_ratio_vs_standard = 0.418`) while reducing its runtime overhead and increasing replicate coverage.
- Preserve the specialized terminal-distribution fast path (`0.492 ms`) as a separate optimization track.
- Improve planner calibration beyond the current measured accuracy of `87.5%` as workload breadth increases.

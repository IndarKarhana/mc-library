# Simulation Techniques Roadmap

This document tracks which Monte Carlo techniques the library supports now, which ones are next, and which newer directions are worth planning for.

## Goals

- improve estimator quality without silently changing target quantities
- add methods that are standard in serious Monte Carlo practice
- prepare the runtime for newer high-performance approaches that matter in modern UQ and quantitative finance

## Current Support

### 1. Standard pseudorandom Monte Carlo

Status:

- supported now

Current implementation:

- deterministic PRNG-backed CPU execution
- fair step-wise path simulation
- specialized terminal-distribution fast path when explicitly applicable

### 2. Antithetic variates

Status:

- supported now for the current European-call CPU runtime

Why it matters:

- it is a simple, well-established variance-reduction technique
- it improves estimator quality without changing the expectation being estimated
- it is often a strong default for symmetric shock-driven path simulation

## Near-Term Techniques

### 1. Scrambled Sobol / randomized quasi-Monte Carlo

Status:

- not implemented yet
- high-priority next technique family

Why it matters:

- low-discrepancy sequences can materially improve convergence on many integration problems
- randomized/scrambled variants preserve statistical error estimation while improving space-filling behavior

Practical notes:

- powers-of-two sample sizing matters for Sobol balance properties
- dimension management and path construction order matter for effectiveness
- this belongs in both CPU and future GPU-oriented sampling plans

Primary sources:

- SciPy Sobol documentation: https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.qmc.Sobol.html
- Pierre L'Ecuyer and Art Owen references cited there, especially scrambling and randomized QMC references

### 2. Latin hypercube sampling

Status:

- not implemented yet
- good candidate for parameter-sweep and uncertainty-propagation workloads

Why it matters:

- stronger space coverage than plain MC for many problems
- useful when inputs are moderately dimensional and nearly additive

Practical notes:

- especially attractive for scientific UQ and sensitivity-analysis style workloads
- often less powerful than scrambled Sobol for the highest-value integration cases, but still broadly useful and user-friendly

Primary source:

- SciPy LatinHypercube documentation: https://docs.scipy.org/doc/scipy-1.16.1/reference/generated/scipy.stats.qmc.LatinHypercube.html

### 3. Control variates

Status:

- not implemented yet
- high-value for workloads with known analytic moments or approximations

Why it matters:

- often gives large variance reduction when a strong correlated control is available
- especially useful in option pricing and calibrated model families

Planned direction:

- start with workload-specific control variates where the reference expectation is known exactly
- later generalize through planner-selected auxiliary statistics

## Advanced High-Value Techniques

### 1. Multilevel Monte Carlo

Status:

- not implemented yet
- strategically important

Why it matters:

- one of the most important modern advances for simulation efficiency
- especially powerful when a hierarchy of discretizations exists
- directly relevant for SDE path simulation and future path-dependent workloads

Primary sources:

- Mike Giles MLMC overview page: https://people.maths.ox.ac.uk/gilesm/mlmc.html
- original 2008 MLMC path simulation paper linked there
- Giles and Waterhouse multilevel quasi-Monte Carlo path simulation reference linked there

### 2. Multilevel randomized quasi-Monte Carlo

Status:

- not implemented yet
- medium-term advanced target

Why it matters:

- combines two of the strongest efficiency ideas available for many workloads
- highly relevant for expensive nested or discretized simulation problems

Implementation note:

- should come after standalone RQMC and MLMC foundations exist

## Emerging / Trending Directions To Track

These are promising, but should follow after the classical high-value techniques above are stable:

- multifidelity multilevel Monte Carlo
- learned or model-assisted control variates
- subset simulation / rare-event specialized methods
- Markov-chain RQMC for specific classes of sequential simulation

These matter, but they should not displace the core roadmap of:

1. standard MC done well
2. strong variance reduction
3. RQMC
4. MLMC
5. calibrated planner support

## Recommended Implementation Order

1. Keep standard and antithetic CPU execution correct and benchmarked.
2. Add explicit technique metadata to user-facing and benchmark-facing outputs.
3. Add scrambled Sobol / randomized QMC support.
4. Add control variates for narrow workloads with strong analytic references.
5. Add MLMC for discretized path simulation.
6. Add MLQMC only after MLMC and RQMC are individually solid.

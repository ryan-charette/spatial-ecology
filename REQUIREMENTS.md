# Research and Implementation Requirements

This document defines the computational scope for Spatial Ecology RS: a reproducible stochastic model of spatial trophic dynamics across connected habitat patches.

## Scientific Objective

Develop a simulation framework for investigating how landscape connectivity, environmental disturbance, and trophic interaction parameters influence ecological stability and extinction risk.

Primary questions:

- How does dispersal among habitat patches change predator-prey-vegetation dynamics?
- How do drought and disease affect time-to-threshold for prey and predator populations?
- Under what parameter regimes does habitat fragmentation amplify or dampen ecological collapse?
- How sensitive are model outcomes to predation, vegetation recovery, and migration assumptions?

## Core Model Requirements

- Represent the landscape as a two-dimensional grid of habitat patches.
- Track prey, predator, vegetation, rainfall, temperature, disease pressure, and carrying capacity per patch.
- Implement discrete-time trophic dynamics with vegetation growth, grazing, prey reproduction, predation, predator reproduction, and natural mortality.
- Implement conservative migration among neighboring patches.
- Support habitat fragmentation by disabling a configurable fraction of patch connections.
- Include stochastic drought, disease, temperature anomaly, and habitat disturbance events.
- Enforce nonnegative finite state variables after each timestep.

## Computational Requirements

- Provide deterministic reproducibility through explicit random seeds.
- Support single-scenario execution from TOML configuration files.
- Support Monte Carlo trials with deterministic per-trial seed derivation.
- Support parameter sweeps over migration rate, drought probability, fragmentation, and predation rate.
- Write patch-level timestep data to CSV.
- Write trial-level summary metrics to CSV.
- Validate numerical stability and migration conservation during execution.

## Required Outputs

Patch-level CSV columns:

```text
trial,timestep,patch_id,row,col,prey,predators,vegetation,rainfall,temperature,event,scenario,seed,disease_pressure
```

Summary CSV columns:

```text
trial,seed,steps,rows,cols,final_prey,final_predators,final_vegetation,
prey_extinct,predator_extinct,time_to_prey_extinction,time_to_predator_extinction,
mean_prey,mean_predators,mean_vegetation,migration_rate,drought_probability,
disease_probability,fragmentation_rate,predation_rate,stability_score,
recovery_time_after_drought,scenario
```

Required figures:

- Mean prey, predator, and vegetation trajectories over time.
- Extinction-risk heatmap across drought probability and migration rate.
- Spatial map of prey abundance at a selected timestep.

## Documentation Requirements

- README with scientific framing, execution commands, experiment design, output schema, and reproducibility notes.
- Model specification documenting state variables, update equations, migration, stochastic events, validation checks, and limitations.
- Scenario files that encode reproducible baseline and sweep experiments.

## Acceptance Criteria

From a clean checkout with Rust installed:

```bash
cargo build --release
cargo test
cargo run --release -- --config configs/baseline.toml --trials 10 --seed 42
cargo run --release -- --config configs/migration_sweep.toml --seed 42
python analysis/plot_population_timeseries.py results/baseline.csv figures/population_timeseries.png
python analysis/plot_extinction_heatmap.py results/sweep_summary.csv figures/extinction_risk_heatmap.png
python analysis/plot_spatial_snapshots.py results/baseline.csv figures/spatial_population_map.png
```

The generated figures must include clear titles, axis labels, legends or colorbars, and enough parameter variation to support scientific interpretation.

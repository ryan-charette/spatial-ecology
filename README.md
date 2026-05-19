# Parallel Spatial Ecosystem Dynamics

Rust-based parallel scientific simulation engine for studying stochastic trophic dynamics across spatially connected habitat patches. The project combines computational ecology with high-performance systems concepts: spatial domain decomposition, worker-thread execution, explicit migration-event routing, deterministic timestep barriers, and benchmark instrumentation.

The repository is organized as a reproducible simulation study and systems benchmark: model equations are documented, scenario parameters are encoded in TOML files, stochastic runs are seeded, serial and parallel modes are comparable under fixed seeds, and outputs are written as analysis-ready CSV files with Python visualization scripts.

![Population time series](figures/population_timeseries.png)

## Scientific Questions

- How does connectivity among habitat patches affect trophic stability?
- Under what drought regimes do prey or predator populations cross functional extinction thresholds?
- Does dispersal buffer local environmental disturbance, or can it synchronize collapse across the landscape?
- How sensitive are extinction outcomes to fragmentation, predation pressure, and vegetation recovery?

## Model Overview

The model is a discrete-time stochastic patch-dynamics system. Each habitat patch tracks:

- prey abundance
- predator abundance
- vegetation biomass
- rainfall
- temperature
- disease pressure
- carrying capacity

At each timestep the engine applies local biological updates, conservative migration between neighboring patches, stochastic environmental disturbance, validation checks, and metrics recording. The landscape uses a von Neumann neighborhood on a 2D grid, with random edge removal controlled by a fragmentation parameter.

The full model specification is in [docs/model_description.md](docs/model_description.md).

## Parallel Execution Model

Habitat patches are partitioned across workers using deterministic contiguous domain decomposition. Each worker owns local patch state for a timestep, applies local ecological updates, and emits `MigrationEvent` records for prey and predator dispersal. A coordinator collects worker messages through Rust channels, sorts and validates events, applies migration deltas, records metrics, and advances the global timestep only after all workers complete the barrier.

```text
                    Coordinator
                         |
        -------------------------------------
        |                 |                 |
     Worker 0          Worker 1          Worker 2
   patch block A     patch block B     patch block C
        |                 |                 |
        -------- migration event routing ----
```

The execution strategy is separate from the ecological model. The same patch dynamics, climate events, and migration rules run in both modes:

```bash
cargo run --release -- --config configs/baseline.toml --mode serial --seed 42
cargo run --release -- --config configs/baseline.toml --mode parallel --workers 4 --seed 42 --validate
```

Parallel reproducibility is enforced with per-patch deterministic RNG streams derived from `(seed, trial, timestep, patch_id)`, so changing `--workers` does not change ecological results for deterministic validation scenarios.

## Reproducible Execution

```bash
cargo build --release
cargo test
cargo run --release -- --config configs/baseline.toml --trials 10 --seed 42
cargo run --release -- --config configs/scaling.toml --mode parallel --workers 4 --benchmark --validate
```

The reference scenario writes:

- `results/baseline.csv`: patch-level state by trial and timestep.
- `results/summary.csv`: one summary row per trial.
- `results/timestep_metrics.csv`: barrier-phase and event metrics by timestep.
- `results/worker_metrics.csv`: worker-local timing and event counts.
- `results/benchmarks/scaling.csv`: benchmark runtime, speedup, and efficiency records.

## Experimental Design

The included scenarios are intended to support small computational experiments rather than one-off demos.

```text
configs/baseline.toml              Reference landscape dynamics
configs/drought_scenario.toml      Elevated drought and disturbance regime
configs/migration_sweep.toml       Drought-connectivity phase space
configs/fragmentation_sweep.toml   Fragmentation and predation sensitivity
```

Run the drought-connectivity sweep:

```bash
cargo run --release -- --config configs/migration_sweep.toml
python analysis/plot_extinction_heatmap.py results/sweep_summary.csv figures/extinction_risk_heatmap.png
```

The sweep varies migration and drought probability while holding fragmentation fixed:

```toml
[sweep]
migration_rates = [0.0, 0.01, 0.03, 0.06, 0.10]
drought_probabilities = [0.0, 0.010, 0.015, 0.020, 0.030]
fragmentation_rates = [0.35]
```

![Extinction risk heatmap](figures/extinction_risk_heatmap.png)

## Outputs

Patch-level output:

```text
trial,timestep,patch_id,row,col,prey,predators,vegetation,rainfall,temperature,event,scenario,seed,disease_pressure
```

Trial-level summary output:

```text
trial,seed,steps,rows,cols,final_prey,final_predators,final_vegetation,
prey_extinct,predator_extinct,time_to_prey_extinction,time_to_predator_extinction,
mean_prey,mean_predators,mean_vegetation,migration_rate,drought_probability,
disease_probability,fragmentation_rate,predation_rate,stability_score,
recovery_time_after_drought,scenario,execution_mode,workers,total_runtime_ms,
mean_timestep_ms,total_events,cross_worker_events,total_edges,local_edges,
boundary_edges,boundary_edge_fraction,patches_per_second,events_per_second
```

The summary table is designed for estimating extinction probabilities, mean time-to-threshold, and comparative sensitivity across parameter settings.

Systems outputs:

```text
results/timestep_metrics.csv
results/worker_metrics.csv
results/benchmarks/scaling.csv
```

The ecological summary includes compact systems fields such as execution mode, workers, runtime, event counts, boundary-edge fraction, and patches per second. Detailed runtime metrics remain in separate CSVs.

## Visualization

```bash
python analysis/plot_population_timeseries.py results/baseline.csv figures/population_timeseries.png
cargo run --release -- --config configs/migration_sweep.toml
python analysis/plot_extinction_heatmap.py results/sweep_summary.csv figures/extinction_risk_heatmap.png
python analysis/plot_spatial_snapshots.py results/baseline.csv figures/spatial_population_map.png
cargo run --release -- --config configs/scaling.toml --mode parallel --workers 4 --benchmark --validate
python analysis/plot_scaling.py results/benchmarks/scaling.csv figures
```

The plotting scripts use Matplotlib when available and fall back to a labeled Pillow renderer when Matplotlib is not installed.

![Spatial population map](figures/spatial_population_map.png)

## Repository Structure

```text
src/
  main.rs          CLI entry point
  config.rs        scenario and sweep configuration parser
  simulation.rs    model initialization and deterministic RNG helpers
  parallel.rs      coordinator-driven timestep barrier
  partition.rs     spatial domain decomposition and boundary metrics
  worker.rs        worker-local patch updates and migration events
  patch.rs         habitat patch state representation
  species.rs       trophic update equations
  climate.rs       stochastic environmental disturbance
  metrics.rs       extinction and stability metrics
  output.rs        CSV writers
  validation.rs    numerical and biological sanity checks
configs/           reproducible scenarios and sweeps
analysis/          Python plotting scripts
docs/              model specification
results/           generated CSV artifacts
figures/           generated figures
```

## Technical Highlights

- Spatial domain decomposition across Rust worker threads.
- Message-passing worker/coordinator execution with deterministic timestep barriers.
- Explicit migration events for cross-partition prey and predator dispersal.
- Patch ownership, event validity, migration conservation, and population-bound validation.
- Serial/parallel equivalence tests under fixed seeds.
- Benchmark CSV and scaling plots for speedup, efficiency, throughput, and communication pressure.

## Reproducibility Notes

All stochastic processes are seeded. For a fixed configuration, trial count, and base seed, the model produces deterministic CSV output. Trial seeds are derived from the base seed, scenario index, and trial index.

The model is intentionally synthetic: parameters are selected to demonstrate computational experimentation with spatial ecological dynamics, not to calibrate a specific empirical ecosystem.

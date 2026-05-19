use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::climate::apply_environment as apply_environmental_events;
use crate::config::{ExecutionMode, RuntimeOptions, SimulationConfig};
use crate::event::event_label;
use crate::metrics::{
    totals_for_patches, MetricsRecorder, SimulationSummary, SystemsSummary, TimestepMetricsRecord,
    WorkerMetricsRecord,
};
use crate::output::OutputWriters;
use crate::partition::PartitionMap;
use crate::patch::Patch;
use crate::simulation::{deterministic_patch_rng, initialize_connectivity, initialize_patches};
use crate::validation::{
    validate_migration, validate_migration_events, validate_partition_map, validate_patches,
};
use crate::worker::{
    CoordinatorToWorker, MigratingSpecies, MigrationEvent, Worker, WorkerStepResult,
    WorkerToCoordinator,
};

#[derive(Clone, Debug)]
struct RoutingStats {
    total_events: usize,
    cross_worker_events: usize,
    incoming_events_by_worker: Vec<usize>,
}

pub fn run_partitioned_trial(
    config: SimulationConfig,
    scenario: String,
    trial: usize,
    seed: u64,
    runtime: &RuntimeOptions,
    mut output: Option<&mut OutputWriters>,
) -> Result<SimulationSummary, String> {
    let effective_workers = runtime.effective_workers();
    let mode = match runtime.mode {
        ExecutionMode::Serial => ExecutionMode::Serial,
        ExecutionMode::Parallel => ExecutionMode::Parallel,
    };
    let mut patches = initialize_patches(&config);
    let connectivity = initialize_connectivity(&config, seed);
    let partition_map = PartitionMap::contiguous(patches.len(), effective_workers);
    if runtime.validate {
        validate_partition_map(&partition_map).map_err(|error| error.to_string())?;
    }
    let boundary_metrics = partition_map.boundary_metrics(&connectivity);
    let mut metrics = MetricsRecorder::new();
    let mut total_events = 0usize;
    let mut cross_worker_events = 0usize;
    let mut timestep_ms_sum = 0.0;
    let run_start = Instant::now();

    metrics.observe(0, &patches, &config);
    if !runtime.benchmark {
        write_timestep(
            &mut output,
            &scenario,
            trial,
            seed,
            0,
            &patches,
            &vec![String::from("initial"); patches.len()],
        )?;
    }

    for timestep in 1..=config.simulation.steps {
        let timestep_start = Instant::now();
        let worker_start = Instant::now();
        let worker_results =
            run_workers(timestep, &patches, &connectivity, &config, &partition_map)?;
        let worker_wall_ms = elapsed_ms(worker_start);

        apply_worker_patch_updates(&mut patches, &worker_results);
        let mut events = collect_events(&worker_results);
        if runtime.validate {
            validate_migration_events(&events, timestep, patches.len())
                .map_err(|error| error.to_string())?;
        }

        let migration_start = Instant::now();
        let routing_stats =
            apply_migration_events(timestep, &mut patches, &mut events, &partition_map)?;
        let migration_apply_ms = elapsed_ms(migration_start);
        total_events += routing_stats.total_events;
        cross_worker_events += routing_stats.cross_worker_events;

        let environment_start = Instant::now();
        let events_for_output = apply_environment(&mut patches, &config, seed, trial, timestep);
        let environment_ms = elapsed_ms(environment_start);

        let validation_start = Instant::now();
        validate_patches(
            &patches,
            config.simulation.rows * config.simulation.cols,
            config.biology.carrying_capacity,
        )
        .map_err(|error| error.to_string())?;
        let validation_ms = elapsed_ms(validation_start);

        let metrics_start = Instant::now();
        metrics.observe(timestep, &patches, &config);
        let metrics_aggregation_ms = elapsed_ms(metrics_start);
        if !runtime.benchmark {
            write_timestep(
                &mut output,
                &scenario,
                trial,
                seed,
                timestep,
                &patches,
                &events_for_output,
            )?;
        }

        let totals = totals_for_patches(&patches);
        let worker_compute_ms = worker_results
            .iter()
            .map(|result| result.local_update_ms + result.event_generation_ms)
            .fold(0.0, f64::max);
        let timestep_ms = elapsed_ms(timestep_start);
        timestep_ms_sum += timestep_ms;
        write_timestep_metrics(
            &mut output,
            TimestepMetricsRecord {
                scenario: scenario.clone(),
                trial,
                timestep,
                mode,
                workers: effective_workers,
                patches: patches.len(),
                total_events: routing_stats.total_events,
                cross_worker_events: routing_stats.cross_worker_events,
                timestep_ms,
                worker_compute_ms,
                event_exchange_ms: (worker_wall_ms - worker_compute_ms).max(0.0),
                migration_apply_ms,
                environment_ms,
                metrics_aggregation_ms,
                validation_ms,
                total_prey: totals.prey,
                total_predators: totals.predators,
                total_vegetation: totals.vegetation,
            },
        )?;
        write_worker_metrics(
            &mut output,
            &scenario,
            trial,
            timestep,
            &worker_results,
            &routing_stats.incoming_events_by_worker,
        )?;
    }

    let total_runtime_ms = elapsed_ms(run_start);
    let mean_timestep_ms = if config.simulation.steps == 0 {
        0.0
    } else {
        timestep_ms_sum / config.simulation.steps as f64
    };
    let patches_per_second = if total_runtime_ms > 0.0 {
        (patches.len() * config.simulation.steps) as f64 / (total_runtime_ms / 1000.0)
    } else {
        0.0
    };
    let events_per_second = if total_runtime_ms > 0.0 {
        total_events as f64 / (total_runtime_ms / 1000.0)
    } else {
        0.0
    };

    let mut summary = metrics.finish(scenario, trial, seed, &config);
    summary.systems = SystemsSummary {
        mode,
        workers: effective_workers,
        total_runtime_ms,
        mean_timestep_ms,
        total_events,
        cross_worker_events,
        total_edges: boundary_metrics.total_edges,
        local_edges: boundary_metrics.local_edges,
        boundary_edges: boundary_metrics.boundary_edges,
        boundary_edge_fraction: boundary_metrics.boundary_fraction,
        patches_per_second,
        events_per_second,
    };

    Ok(summary)
}

fn run_workers(
    timestep: usize,
    patches: &[Patch],
    connectivity: &[Vec<(usize, f64)>],
    config: &SimulationConfig,
    partition_map: &PartitionMap,
) -> Result<Vec<WorkerStepResult>, String> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(partition_map.worker_count());

    for partition in partition_map.partitions() {
        let owned_patches = partition
            .patch_ids
            .iter()
            .map(|patch_id| patches[patch_id.0].clone())
            .collect::<Vec<_>>();
        let connectivity = connectivity.to_vec();
        let config = config.clone();
        let tx = tx.clone();
        let worker_id = partition.id;
        let command = CoordinatorToWorker::Step { timestep };
        handles.push(thread::spawn(move || {
            let CoordinatorToWorker::Step { timestep } = command;
            let worker = Worker::new(worker_id, owned_patches, connectivity, config);
            let result = worker.run_timestep(timestep);
            tx.send(WorkerToCoordinator::StepComplete {
                worker_id,
                timestep,
                result,
            })
            .map_err(|error| format!("failed to send worker result: {error}"))
        }));
    }
    drop(tx);

    let mut results = Vec::with_capacity(partition_map.worker_count());
    for _ in 0..partition_map.worker_count() {
        match rx
            .recv()
            .map_err(|error| format!("failed to receive worker result: {error}"))?
        {
            WorkerToCoordinator::StepComplete {
                worker_id,
                timestep: result_timestep,
                result,
            } => {
                if result_timestep != timestep {
                    return Err(format!(
                        "worker {worker_id} reported timestep {result_timestep} during timestep {timestep}"
                    ));
                }
                results.push(result);
            }
        }
    }

    for handle in handles {
        handle
            .join()
            .map_err(|_| String::from("worker thread panicked"))?
            .map_err(|error| error)?;
    }

    results.sort_by_key(|result| result.worker_id);
    Ok(results)
}

fn apply_worker_patch_updates(patches: &mut [Patch], worker_results: &[WorkerStepResult]) {
    for result in worker_results {
        for patch in &result.patches {
            patches[patch.id.0] = patch.clone();
        }
    }
}

fn collect_events(worker_results: &[WorkerStepResult]) -> Vec<MigrationEvent> {
    let mut events = Vec::new();
    for result in worker_results {
        events.extend(result.events.iter().cloned());
    }
    events
}

fn apply_migration_events(
    timestep: usize,
    patches: &mut [Patch],
    events: &mut [MigrationEvent],
    partition_map: &PartitionMap,
) -> Result<RoutingStats, String> {
    events.sort_by_key(|event| {
        (
            event.source_patch.0,
            event.destination_patch.0,
            event.species,
        )
    });

    let before = totals_for_patches(patches);
    let mut prey_delta = vec![0.0; patches.len()];
    let mut predator_delta = vec![0.0; patches.len()];
    let mut incoming_events_by_worker = vec![0usize; partition_map.worker_count()];
    let mut cross_worker_events = 0usize;

    let event_count = events.len();
    for event in events.iter() {
        if event.timestep != timestep {
            return Err(format!(
                "coordinator received event for timestep {} during timestep {}",
                event.timestep, timestep
            ));
        }

        let source_owner = partition_map
            .owner(event.source_patch)
            .ok_or_else(|| format!("source patch {} has no owner", event.source_patch.0))?;
        let destination_owner = partition_map
            .owner(event.destination_patch)
            .ok_or_else(|| {
                format!(
                    "destination patch {} has no owner",
                    event.destination_patch.0
                )
            })?;
        incoming_events_by_worker[destination_owner] += 1;
        if source_owner != destination_owner {
            cross_worker_events += 1;
        }

        match event.species {
            MigratingSpecies::Prey => {
                prey_delta[event.source_patch.0] -= event.amount;
                prey_delta[event.destination_patch.0] += event.amount;
            }
            MigratingSpecies::Predators => {
                predator_delta[event.source_patch.0] -= event.amount;
                predator_delta[event.destination_patch.0] += event.amount;
            }
        }
    }

    for (index, patch) in patches.iter_mut().enumerate() {
        patch.state.prey += prey_delta[index];
        patch.state.predators += predator_delta[index];
        patch.state.clamp_nonnegative();
    }

    let after = totals_for_patches(patches);
    validate_migration(
        before.prey,
        after.prey,
        before.predators,
        after.predators,
        1.0e-6,
    )
    .map_err(|error| error.to_string())?;

    Ok(RoutingStats {
        total_events: event_count,
        cross_worker_events,
        incoming_events_by_worker,
    })
}

fn apply_environment(
    patches: &mut [Patch],
    config: &SimulationConfig,
    seed: u64,
    trial: usize,
    timestep: usize,
) -> Vec<String> {
    patches
        .iter_mut()
        .map(|patch| {
            let mut rng = deterministic_patch_rng(seed, trial, timestep, patch.id.0);
            let events =
                apply_environmental_events(&mut patch.state, &config.environment, &mut rng);
            event_label(&events)
        })
        .collect()
}

fn write_timestep(
    output: &mut Option<&mut OutputWriters>,
    scenario: &str,
    trial: usize,
    seed: u64,
    timestep: usize,
    patches: &[Patch],
    events: &[String],
) -> Result<(), String> {
    if let Some(writer) = output.as_deref_mut() {
        for (patch, event) in patches.iter().zip(events.iter()) {
            writer.write_patch_record(scenario, trial, seed, timestep, patch, event)?;
        }
    }

    Ok(())
}

fn write_timestep_metrics(
    output: &mut Option<&mut OutputWriters>,
    record: TimestepMetricsRecord,
) -> Result<(), String> {
    if let Some(writer) = output.as_deref_mut() {
        writer.write_timestep_metrics(&record)?;
    }

    Ok(())
}

fn write_worker_metrics(
    output: &mut Option<&mut OutputWriters>,
    scenario: &str,
    trial: usize,
    timestep: usize,
    worker_results: &[WorkerStepResult],
    incoming_events_by_worker: &[usize],
) -> Result<(), String> {
    if let Some(writer) = output.as_deref_mut() {
        for result in worker_results {
            let owned_patches = result.patches.len();
            let divisor = owned_patches.max(1) as f64;
            writer.write_worker_metrics(&WorkerMetricsRecord {
                scenario: scenario.to_string(),
                trial,
                timestep,
                worker_id: result.worker_id,
                owned_patches,
                outgoing_events: result.events.len(),
                incoming_events: incoming_events_by_worker[result.worker_id],
                local_update_ms: result.local_update_ms,
                event_generation_ms: result.event_generation_ms,
                mean_prey: result.totals_after_local_update.prey / divisor,
                mean_predators: result.totals_after_local_update.predators / divisor,
                mean_vegetation: result.totals_after_local_update.vegetation / divisor,
            })?;
        }
    }

    Ok(())
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::run_partitioned_trial;
    use crate::config::{ExecutionMode, RuntimeOptions, SimulationConfig};

    #[test]
    fn serial_and_parallel_runs_are_numerically_equivalent() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 4;
        config.simulation.cols = 4;
        config.simulation.steps = 12;
        config.environment.drought_probability = 0.05;
        config.environment.disease_probability = 0.02;

        let serial = run_partitioned_trial(
            config.clone(),
            String::from("equivalence"),
            0,
            11,
            &RuntimeOptions {
                mode: ExecutionMode::Serial,
                workers: 1,
                benchmark: false,
                validate: true,
            },
            None,
        )
        .expect("serial run should succeed");

        let parallel = run_partitioned_trial(
            config,
            String::from("equivalence"),
            0,
            11,
            &RuntimeOptions {
                mode: ExecutionMode::Parallel,
                workers: 4,
                benchmark: false,
                validate: true,
            },
            None,
        )
        .expect("parallel run should succeed");

        assert!((serial.final_prey - parallel.final_prey).abs() < 1.0e-9);
        assert!((serial.final_predators - parallel.final_predators).abs() < 1.0e-9);
        assert!((serial.final_vegetation - parallel.final_vegetation).abs() < 1.0e-9);
    }

    #[test]
    #[ignore]
    fn large_parallel_simulation_stress_test() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 100;
        config.simulation.cols = 100;
        config.simulation.steps = 250;
        config.simulation.trials = 1;

        run_partitioned_trial(
            config,
            String::from("large_parallel_stress"),
            0,
            123,
            &RuntimeOptions {
                mode: ExecutionMode::Parallel,
                workers: 8,
                benchmark: true,
                validate: true,
            },
            None,
        )
        .expect("large parallel run should complete");
    }
}

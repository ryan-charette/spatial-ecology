use crate::config::{ExecutionMode, RuntimeOptions, SimulationConfig};
use crate::output::OutputWriters;
use crate::parallel::run_partitioned_trial;

#[derive(Clone, Debug)]
pub struct ExperimentReport {
    pub scenarios: usize,
    pub trials_per_scenario: usize,
    pub results_csv: String,
    pub summary_csv: String,
    pub mode: ExecutionMode,
    pub workers: usize,
}

pub fn run_experiments(
    config: SimulationConfig,
    runtime: RuntimeOptions,
) -> Result<ExperimentReport, String> {
    let scenarios = config.scenario_configs();
    let trials = config.simulation.trials;
    let scenario_count = scenarios.len();
    let results_csv = config.output.results_csv.clone();
    let summary_csv = config.output.summary_csv.clone();
    let mut output = OutputWriters::create(&config.output, runtime.benchmark)?;

    for (scenario_index, scenario_config) in scenarios.into_iter().enumerate() {
        for trial in 0..trials {
            let seed = scenario_seed(scenario_config.simulation.seed, scenario_index, trial);
            let mut trial_config = scenario_config.clone();
            trial_config.simulation.seed = seed;

            let benchmark_baseline = if runtime.benchmark
                && runtime.mode == ExecutionMode::Parallel
                && runtime.workers > 1
            {
                let mut serial_runtime = runtime.clone();
                serial_runtime.mode = ExecutionMode::Serial;
                serial_runtime.workers = 1;
                Some(run_partitioned_trial(
                    trial_config.clone(),
                    scenario_config.scenario_name.clone(),
                    trial,
                    seed,
                    &serial_runtime,
                    None,
                )?)
            } else {
                None
            };

            let summary = run_partitioned_trial(
                trial_config,
                scenario_config.scenario_name.clone(),
                trial,
                seed,
                &runtime,
                Some(&mut output),
            )?;

            if runtime.benchmark {
                if let Some(serial_summary) = &benchmark_baseline {
                    output.write_benchmark_record(
                        serial_summary,
                        Some(serial_summary.systems.total_runtime_ms),
                    )?;
                    output.write_benchmark_record(
                        &summary,
                        Some(serial_summary.systems.total_runtime_ms),
                    )?;
                } else {
                    output
                        .write_benchmark_record(&summary, Some(summary.systems.total_runtime_ms))?;
                }
            }

            output.write_summary(&summary)?;
        }
    }

    output.flush()?;

    Ok(ExperimentReport {
        scenarios: scenario_count,
        trials_per_scenario: trials,
        results_csv,
        summary_csv,
        mode: runtime.mode,
        workers: runtime.effective_workers(),
    })
}

fn scenario_seed(base_seed: u64, scenario_index: usize, trial: usize) -> u64 {
    base_seed
        .wrapping_add((scenario_index as u64).wrapping_mul(1_000_003))
        .wrapping_add((trial as u64).wrapping_mul(9_973))
}

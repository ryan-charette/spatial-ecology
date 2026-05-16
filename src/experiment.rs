use crate::config::SimulationConfig;
use crate::output::OutputWriters;
use crate::simulation::Simulation;

#[derive(Clone, Debug)]
pub struct ExperimentReport {
    pub scenarios: usize,
    pub trials_per_scenario: usize,
    pub results_csv: String,
    pub summary_csv: String,
}

pub fn run_experiments(config: SimulationConfig) -> Result<ExperimentReport, String> {
    let scenarios = config.scenario_configs();
    let trials = config.simulation.trials;
    let results_csv = config.output.results_csv.clone();
    let summary_csv = config.output.summary_csv.clone();
    let mut output = OutputWriters::create(&config.output)?;

    for (scenario_index, scenario_config) in scenarios.into_iter().enumerate() {
        for trial in 0..trials {
            let seed = scenario_seed(scenario_config.simulation.seed, scenario_index, trial);
            let mut trial_config = scenario_config.clone();
            trial_config.simulation.seed = seed;

            let mut simulation = Simulation::new(
                trial_config,
                scenario_config.scenario_name.clone(),
                trial,
                seed,
            );
            let summary = simulation.run(&mut output)?;
            output.write_summary(&summary)?;
        }
    }

    output.flush()?;

    Ok(ExperimentReport {
        scenarios: config.scenario_configs().len(),
        trials_per_scenario: trials,
        results_csv,
        summary_csv,
    })
}

fn scenario_seed(base_seed: u64, scenario_index: usize, trial: usize) -> u64 {
    base_seed
        .wrapping_add((scenario_index as u64).wrapping_mul(1_000_003))
        .wrapping_add((trial as u64).wrapping_mul(9_973))
}

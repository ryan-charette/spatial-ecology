use crate::config::SimulationConfig;
use crate::patch::Patch;

#[derive(Clone, Copy, Debug, Default)]
pub struct PopulationTotals {
    pub prey: f64,
    pub predators: f64,
    pub vegetation: f64,
}

#[derive(Clone, Debug)]
pub struct TotalSample {
    pub totals: PopulationTotals,
}

#[derive(Clone, Debug)]
pub struct MetricsRecorder {
    samples: Vec<TotalSample>,
    time_to_prey_extinction: Option<usize>,
    time_to_predator_extinction: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct SimulationSummary {
    pub scenario: String,
    pub trial: usize,
    pub seed: u64,
    pub steps: usize,
    pub rows: usize,
    pub cols: usize,
    pub final_prey: f64,
    pub final_predators: f64,
    pub final_vegetation: f64,
    pub prey_extinct: bool,
    pub predator_extinct: bool,
    pub time_to_prey_extinction: Option<usize>,
    pub time_to_predator_extinction: Option<usize>,
    pub mean_prey: f64,
    pub mean_predators: f64,
    pub mean_vegetation: f64,
    pub migration_rate: f64,
    pub drought_probability: f64,
    pub disease_probability: f64,
    pub fragmentation_rate: f64,
    pub predation_rate: f64,
    pub stability_score: f64,
    pub recovery_time_after_drought: Option<usize>,
}

impl MetricsRecorder {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            time_to_prey_extinction: None,
            time_to_predator_extinction: None,
        }
    }

    pub fn observe(&mut self, timestep: usize, patches: &[Patch], config: &SimulationConfig) {
        let totals = totals_for_patches(patches);

        if self.time_to_prey_extinction.is_none()
            && totals.prey <= config.thresholds.prey_extinction_threshold
        {
            self.time_to_prey_extinction = Some(timestep);
        }

        if self.time_to_predator_extinction.is_none()
            && totals.predators <= config.thresholds.predator_extinction_threshold
        {
            self.time_to_predator_extinction = Some(timestep);
        }

        self.samples.push(TotalSample { totals });
    }

    pub fn finish(
        &self,
        scenario: String,
        trial: usize,
        seed: u64,
        config: &SimulationConfig,
    ) -> SimulationSummary {
        let final_totals = self
            .samples
            .last()
            .map(|sample| sample.totals)
            .unwrap_or_default();

        let sample_count = self.samples.len().max(1) as f64;
        let sum = self
            .samples
            .iter()
            .fold(PopulationTotals::default(), |mut acc, sample| {
                acc.prey += sample.totals.prey;
                acc.predators += sample.totals.predators;
                acc.vegetation += sample.totals.vegetation;
                acc
            });

        let mean_prey = sum.prey / sample_count;
        let mean_predators = sum.predators / sample_count;
        let mean_vegetation = sum.vegetation / sample_count;
        let stability_score = stability_score(&self.samples, mean_prey);

        SimulationSummary {
            scenario,
            trial,
            seed,
            steps: config.simulation.steps,
            rows: config.simulation.rows,
            cols: config.simulation.cols,
            final_prey: final_totals.prey,
            final_predators: final_totals.predators,
            final_vegetation: final_totals.vegetation,
            prey_extinct: self.time_to_prey_extinction.is_some(),
            predator_extinct: self.time_to_predator_extinction.is_some(),
            time_to_prey_extinction: self.time_to_prey_extinction,
            time_to_predator_extinction: self.time_to_predator_extinction,
            mean_prey,
            mean_predators,
            mean_vegetation,
            migration_rate: config.migration.prey_migration_rate,
            drought_probability: config.environment.drought_probability,
            disease_probability: config.environment.disease_probability,
            fragmentation_rate: config.migration.fragmentation_rate,
            predation_rate: config.biology.predation_rate,
            stability_score,
            recovery_time_after_drought: None,
        }
    }
}

pub fn totals_for_patches(patches: &[Patch]) -> PopulationTotals {
    patches
        .iter()
        .fold(PopulationTotals::default(), |mut acc, patch| {
            acc.prey += patch.state.prey;
            acc.predators += patch.state.predators;
            acc.vegetation += patch.state.vegetation;
            acc
        })
}

fn stability_score(samples: &[TotalSample], mean_prey: f64) -> f64 {
    if samples.len() < 2 || mean_prey <= f64::EPSILON {
        return 1.0;
    }

    let variance = samples
        .iter()
        .map(|sample| {
            let diff = sample.totals.prey - mean_prey;
            diff * diff
        })
        .sum::<f64>()
        / samples.len() as f64;
    let coefficient_of_variation = variance.sqrt() / mean_prey.max(1.0e-9);
    1.0 / (1.0 + coefficient_of_variation)
}

use crate::climate::apply_environment as apply_environmental_events;
use crate::config::SimulationConfig;
use crate::event::{event_label, SmallRng};
use crate::metrics::{totals_for_patches, MetricsRecorder, SimulationSummary};
use crate::output::OutputWriters;
use crate::patch::{Patch, PatchState};
use crate::species::update_patch_state;
use crate::validation::{validate_migration, validate_patches};

pub struct Simulation {
    config: SimulationConfig,
    scenario: String,
    trial: usize,
    seed: u64,
    patches: Vec<Patch>,
    connectivity: Vec<Vec<(usize, f64)>>,
    rng: SmallRng,
    metrics: MetricsRecorder,
}

impl Simulation {
    pub fn new(config: SimulationConfig, scenario: String, trial: usize, seed: u64) -> Self {
        let mut rng = SmallRng::new(seed);
        let patches = initialize_patches(&config);
        let connectivity = initialize_connectivity(&config, &mut rng);

        Self {
            config,
            scenario,
            trial,
            seed,
            patches,
            connectivity,
            rng,
            metrics: MetricsRecorder::new(),
        }
    }

    pub fn run(&mut self, output: &mut OutputWriters) -> Result<SimulationSummary, String> {
        let patch_count = self.patches.len();
        self.metrics.observe(0, &self.patches, &self.config);
        self.write_timestep(output, 0, &vec![String::from("initial"); patch_count])?;

        for timestep in 1..=self.config.simulation.steps {
            self.step(timestep)?;
            let events = self.apply_environment();
            validate_patches(
                &self.patches,
                self.config.simulation.rows * self.config.simulation.cols,
                self.config.biology.carrying_capacity,
            )
            .map_err(|error| error.to_string())?;
            self.metrics.observe(timestep, &self.patches, &self.config);
            self.write_timestep(output, timestep, &events)?;
        }

        Ok(self
            .metrics
            .finish(self.scenario.clone(), self.trial, self.seed, &self.config))
    }

    fn step(&mut self, _timestep: usize) -> Result<(), String> {
        for patch in &mut self.patches {
            update_patch_state(&mut patch.state, &self.config.biology);
        }

        self.apply_migration()?;

        validate_patches(
            &self.patches,
            self.config.simulation.rows * self.config.simulation.cols,
            self.config.biology.carrying_capacity,
        )
        .map_err(|error| error.to_string())
    }

    fn apply_migration(&mut self) -> Result<(), String> {
        let before = totals_for_patches(&self.patches);
        let mut prey_delta = vec![0.0; self.patches.len()];
        let mut predator_delta = vec![0.0; self.patches.len()];

        for (from, edges) in self.connectivity.iter().enumerate() {
            if edges.is_empty() {
                continue;
            }

            let state = &self.patches[from].state;
            let total_weight = edges.iter().map(|(_, weight)| *weight).sum::<f64>();
            if total_weight <= 0.0 {
                continue;
            }

            let forage_ratio = state.vegetation / state.carrying_capacity.max(1.0);
            let prey_rate = adjusted_migration_rate(
                self.config.migration.prey_migration_rate,
                forage_ratio,
                self.config.migration.scarcity_threshold,
                self.config.migration.scarcity_migration_multiplier,
            );
            let predator_food_ratio = if state.predators <= f64::EPSILON {
                1.0
            } else {
                state.prey / (state.predators * 5.0).max(1.0)
            };
            let predator_rate = adjusted_migration_rate(
                self.config.migration.predator_migration_rate,
                predator_food_ratio,
                self.config.migration.scarcity_threshold,
                self.config.migration.scarcity_migration_multiplier,
            );

            let prey_migrants = state.prey * prey_rate;
            let predator_migrants = state.predators * predator_rate;
            prey_delta[from] -= prey_migrants;
            predator_delta[from] -= predator_migrants;

            for (to, weight) in edges {
                let share = *weight / total_weight;
                prey_delta[*to] += prey_migrants * share;
                predator_delta[*to] += predator_migrants * share;
            }
        }

        for (index, patch) in self.patches.iter_mut().enumerate() {
            patch.state.prey += prey_delta[index];
            patch.state.predators += predator_delta[index];
            patch.state.clamp_nonnegative();
        }

        let after = totals_for_patches(&self.patches);
        validate_migration(
            before.prey,
            after.prey,
            before.predators,
            after.predators,
            1.0e-6,
        )
        .map_err(|error| error.to_string())
    }

    fn apply_environment(&mut self) -> Vec<String> {
        self.patches
            .iter_mut()
            .map(|patch| {
                let events = apply_environmental_events(
                    &mut patch.state,
                    &self.config.environment,
                    &mut self.rng,
                );
                event_label(&events)
            })
            .collect()
    }

    fn write_timestep(
        &self,
        output: &mut OutputWriters,
        timestep: usize,
        events: &[String],
    ) -> Result<(), String> {
        for (patch, event) in self.patches.iter().zip(events.iter()) {
            output.write_patch_record(
                &self.scenario,
                self.trial,
                self.seed,
                timestep,
                patch,
                event,
            )?;
        }

        Ok(())
    }
}

fn initialize_patches(config: &SimulationConfig) -> Vec<Patch> {
    let mut patches = Vec::with_capacity(config.simulation.rows * config.simulation.cols);

    for row in 0..config.simulation.rows {
        for col in 0..config.simulation.cols {
            let id = row * config.simulation.cols + col;
            let state = PatchState {
                prey: config.initial_conditions.prey,
                predators: config.initial_conditions.predators,
                vegetation: config.initial_conditions.vegetation,
                rainfall: config.initial_conditions.rainfall,
                temperature: config.initial_conditions.temperature,
                disease_pressure: config.initial_conditions.disease_pressure,
                carrying_capacity: config.biology.carrying_capacity,
            };
            patches.push(Patch::new(id, row, col, state));
        }
    }

    patches
}

fn initialize_connectivity(
    config: &SimulationConfig,
    rng: &mut SmallRng,
) -> Vec<Vec<(usize, f64)>> {
    let rows = config.simulation.rows;
    let cols = config.simulation.cols;
    let mut connectivity = vec![Vec::new(); rows * cols];

    for row in 0..rows {
        for col in 0..cols {
            let from = row * cols + col;

            if col + 1 < cols {
                add_edge_if_connected(from, row * cols + col + 1, &mut connectivity, config, rng);
            }

            if row + 1 < rows {
                add_edge_if_connected(from, (row + 1) * cols + col, &mut connectivity, config, rng);
            }
        }
    }

    connectivity
}

fn add_edge_if_connected(
    a: usize,
    b: usize,
    connectivity: &mut [Vec<(usize, f64)>],
    config: &SimulationConfig,
    rng: &mut SmallRng,
) {
    if rng.chance(config.migration.fragmentation_rate) {
        return;
    }

    connectivity[a].push((b, 1.0));
    connectivity[b].push((a, 1.0));
}

fn adjusted_migration_rate(
    base_rate: f64,
    resource_ratio: f64,
    threshold: f64,
    multiplier: f64,
) -> f64 {
    let mut rate = base_rate.clamp(0.0, 1.0);
    if resource_ratio < threshold {
        rate *= multiplier.max(0.0);
    }

    rate.clamp(0.0, 0.95)
}

#[cfg(test)]
mod tests {
    use super::Simulation;
    use crate::config::SimulationConfig;
    use crate::metrics::totals_for_patches;

    #[test]
    fn migration_conserves_animals() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 2;
        config.simulation.cols = 2;
        config.migration.fragmentation_rate = 0.0;
        config.environment.drought_probability = 0.0;
        config.environment.disease_probability = 0.0;

        let mut simulation = Simulation::new(config, String::from("test"), 0, 7);
        let before = totals_for_patches(&simulation.patches);
        simulation.apply_migration().expect("migration should pass");
        let after = totals_for_patches(&simulation.patches);

        assert!((before.prey - after.prey).abs() < 1.0e-9);
        assert!((before.predators - after.predators).abs() < 1.0e-9);
    }
}

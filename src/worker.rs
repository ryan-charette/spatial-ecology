use std::time::Instant;

use crate::config::SimulationConfig;
use crate::metrics::{totals_for_patches, PopulationTotals};
use crate::patch::{Patch, PatchId};
use crate::simulation::adjusted_migration_rate;
use crate::species::update_patch_state;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MigratingSpecies {
    Prey,
    Predators,
}

#[derive(Clone, Debug)]
pub struct MigrationEvent {
    pub timestep: usize,
    pub source_patch: PatchId,
    pub destination_patch: PatchId,
    pub species: MigratingSpecies,
    pub amount: f64,
}

#[derive(Clone, Debug)]
pub struct WorkerStepResult {
    pub worker_id: usize,
    pub patches: Vec<Patch>,
    pub events: Vec<MigrationEvent>,
    pub local_update_ms: f64,
    pub event_generation_ms: f64,
    pub totals_after_local_update: PopulationTotals,
}

#[derive(Clone, Debug)]
pub enum CoordinatorToWorker {
    Step { timestep: usize },
}

#[derive(Clone, Debug)]
pub enum WorkerToCoordinator {
    StepComplete {
        worker_id: usize,
        timestep: usize,
        result: WorkerStepResult,
    },
}

pub struct Worker {
    id: usize,
    patches: Vec<Patch>,
    connectivity: Vec<Vec<(usize, f64)>>,
    config: SimulationConfig,
}

impl Worker {
    pub fn new(
        id: usize,
        patches: Vec<Patch>,
        connectivity: Vec<Vec<(usize, f64)>>,
        config: SimulationConfig,
    ) -> Self {
        Self {
            id,
            patches,
            connectivity,
            config,
        }
    }

    pub fn run_timestep(mut self, timestep: usize) -> WorkerStepResult {
        let update_start = Instant::now();
        for patch in &mut self.patches {
            update_patch_state(&mut patch.state, &self.config.biology);
        }
        let local_update_ms = elapsed_ms(update_start);
        let totals_after_local_update = totals_for_patches(&self.patches);

        let event_start = Instant::now();
        let mut events = Vec::new();
        for patch in &self.patches {
            generate_patch_events(
                timestep,
                patch,
                &self.connectivity[patch.id.0],
                &self.config,
                &mut events,
            );
        }
        let event_generation_ms = elapsed_ms(event_start);

        WorkerStepResult {
            worker_id: self.id,
            patches: self.patches,
            events,
            local_update_ms,
            event_generation_ms,
            totals_after_local_update,
        }
    }
}

fn generate_patch_events(
    timestep: usize,
    patch: &Patch,
    edges: &[(usize, f64)],
    config: &SimulationConfig,
    events: &mut Vec<MigrationEvent>,
) {
    if edges.is_empty() {
        return;
    }

    let total_weight = edges.iter().map(|(_, weight)| *weight).sum::<f64>();
    if total_weight <= 0.0 {
        return;
    }

    let state = &patch.state;
    let forage_ratio = state.vegetation / state.carrying_capacity.max(1.0);
    let prey_rate = adjusted_migration_rate(
        config.migration.prey_migration_rate,
        forage_ratio,
        config.migration.scarcity_threshold,
        config.migration.scarcity_migration_multiplier,
    );
    let predator_food_ratio = if state.predators <= f64::EPSILON {
        1.0
    } else {
        state.prey / (state.predators * 5.0).max(1.0)
    };
    let predator_rate = adjusted_migration_rate(
        config.migration.predator_migration_rate,
        predator_food_ratio,
        config.migration.scarcity_threshold,
        config.migration.scarcity_migration_multiplier,
    );

    let prey_migrants = state.prey * prey_rate;
    let predator_migrants = state.predators * predator_rate;

    for (destination, weight) in edges {
        let share = *weight / total_weight;
        let destination_patch = PatchId(*destination);
        events.push(MigrationEvent {
            timestep,
            source_patch: patch.id,
            destination_patch,
            species: MigratingSpecies::Prey,
            amount: prey_migrants * share,
        });
        events.push(MigrationEvent {
            timestep,
            source_patch: patch.id,
            destination_patch,
            species: MigratingSpecies::Predators,
            amount: predator_migrants * share,
        });
    }
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::{MigratingSpecies, Worker};
    use crate::config::SimulationConfig;
    use crate::simulation::{initialize_connectivity, initialize_patches};

    #[test]
    fn worker_generates_species_migration_events() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 1;
        config.simulation.cols = 2;
        config.migration.fragmentation_rate = 0.0;
        let patches = initialize_patches(&config);
        let connectivity = initialize_connectivity(&config, 5);
        let worker = Worker::new(0, patches, connectivity, config);
        let result = worker.run_timestep(1);

        assert!(result
            .events
            .iter()
            .any(|event| event.species == MigratingSpecies::Prey));
        assert!(result
            .events
            .iter()
            .any(|event| event.species == MigratingSpecies::Predators));
    }
}

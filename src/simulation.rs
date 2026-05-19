use crate::config::SimulationConfig;
use crate::event::SmallRng;
use crate::patch::{Patch, PatchState};

pub fn initialize_patches(config: &SimulationConfig) -> Vec<Patch> {
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

pub fn initialize_connectivity(config: &SimulationConfig, seed: u64) -> Vec<Vec<(usize, f64)>> {
    let rows = config.simulation.rows;
    let cols = config.simulation.cols;
    let mut connectivity = vec![Vec::new(); rows * cols];
    let mut rng = SmallRng::new(seed ^ 0x9e37_79b9_7f4a_7c15);

    for row in 0..rows {
        for col in 0..cols {
            let from = row * cols + col;

            if col + 1 < cols {
                add_edge_if_connected(
                    from,
                    row * cols + col + 1,
                    &mut connectivity,
                    config,
                    &mut rng,
                );
            }

            if row + 1 < rows {
                add_edge_if_connected(
                    from,
                    (row + 1) * cols + col,
                    &mut connectivity,
                    config,
                    &mut rng,
                );
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

pub fn adjusted_migration_rate(
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

pub fn deterministic_patch_rng(
    seed: u64,
    trial: usize,
    timestep: usize,
    patch_id: usize,
) -> SmallRng {
    SmallRng::new(
        seed ^ mix_u64(trial as u64).rotate_left(13)
            ^ mix_u64(timestep as u64).rotate_left(29)
            ^ mix_u64(patch_id as u64).rotate_left(43),
    )
}

fn mix_u64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{deterministic_patch_rng, initialize_connectivity, initialize_patches};
    use crate::config::SimulationConfig;

    #[test]
    fn initialization_creates_expected_patch_grid() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 2;
        config.simulation.cols = 3;

        let patches = initialize_patches(&config);

        assert_eq!(patches.len(), 6);
        assert_eq!(patches[5].coord.row, 1);
        assert_eq!(patches[5].coord.col, 2);
    }

    #[test]
    fn connectivity_is_seed_reproducible() {
        let mut config = SimulationConfig::default();
        config.simulation.rows = 3;
        config.simulation.cols = 3;
        config.migration.fragmentation_rate = 0.0;

        let first = initialize_connectivity(&config, 10);
        let second = initialize_connectivity(&config, 10);

        assert_eq!(first, second);
    }

    #[test]
    fn patch_rng_is_independent_by_patch_and_timestep() {
        let mut a = deterministic_patch_rng(42, 0, 1, 5);
        let mut b = deterministic_patch_rng(42, 0, 1, 5);
        let mut c = deterministic_patch_rng(42, 0, 2, 5);

        assert_eq!(a.next_f64(), b.next_f64());
        assert_ne!(a.next_f64(), c.next_f64());
    }
}

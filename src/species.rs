use crate::config::BiologyConfig;
use crate::patch::PatchState;

pub fn update_patch_state(state: &mut PatchState, biology: &BiologyConfig) {
    let carrying_capacity = biology.carrying_capacity.max(1.0);
    let vegetation = state.vegetation.max(0.0);
    let prey = state.prey.max(0.0);
    let predators = state.predators.max(0.0);

    let crowding = (1.0 - vegetation / carrying_capacity).max(0.0);
    let rainfall_factor = state.rainfall.clamp(0.25, 1.50);
    let temperature_stress = ((state.temperature - biology.optimal_temperature).abs()
        / biology.temperature_tolerance)
        .clamp(0.0, 0.75);
    let disease_stress = (1.0 - 0.35 * state.disease_pressure.clamp(0.0, 1.0)).max(0.0);

    let vegetation_growth = biology.vegetation_growth_rate
        * vegetation
        * crowding
        * rainfall_factor
        * (1.0 - temperature_stress);
    let grazing = (biology.vegetation_grazing_rate * prey * vegetation / carrying_capacity)
        .min(vegetation + vegetation_growth);
    let forage_factor = (vegetation / carrying_capacity).clamp(0.0, 1.25);
    let prey_births = biology.prey_birth_rate * prey * forage_factor * disease_stress;
    let prey_deaths = biology.prey_death_rate * prey;
    let predation = (biology.predation_rate * prey * predators).min(prey + prey_births);
    let predator_births = biology.predator_conversion_efficiency * predation;
    let predator_deaths = biology.predator_death_rate * predators;

    state.vegetation = vegetation + vegetation_growth - grazing;
    state.prey = prey + prey_births - predation - prey_deaths;
    state.predators = predators + predator_births - predator_deaths;
    state.vegetation = state.vegetation.min(carrying_capacity);
    state.clamp_nonnegative();
}

#[cfg(test)]
mod tests {
    use super::update_patch_state;
    use crate::config::BiologyConfig;
    use crate::patch::PatchState;

    #[test]
    fn patch_update_keeps_state_nonnegative() {
        let mut state = PatchState {
            prey: 1.0,
            predators: 500.0,
            vegetation: 1.0,
            rainfall: 1.0,
            temperature: 20.0,
            disease_pressure: 0.0,
            carrying_capacity: 1000.0,
        };

        update_patch_state(&mut state, &BiologyConfig::default());
        assert!(state.prey >= 0.0);
        assert!(state.predators >= 0.0);
        assert!(state.vegetation >= 0.0);
    }
}

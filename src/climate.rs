use crate::config::EnvironmentConfig;
use crate::event::{EcologicalEvent, SmallRng};
use crate::patch::PatchState;

pub fn apply_environment(
    state: &mut PatchState,
    environment: &EnvironmentConfig,
    rng: &mut SmallRng,
) -> Vec<EcologicalEvent> {
    let mut events = Vec::new();

    if rng.chance(environment.drought_probability) {
        state.vegetation *= 1.0 - environment.drought_vegetation_loss.clamp(0.0, 1.0);
        state.rainfall *= 0.65;
        events.push(EcologicalEvent::Drought);
    } else {
        state.rainfall = (state.rainfall * 0.95) + 0.05;
    }

    if rng.chance(environment.disease_probability) {
        state.prey *= 1.0 - environment.disease_prey_mortality.clamp(0.0, 1.0);
        state.disease_pressure = (state.disease_pressure + 0.5).clamp(0.0, 1.0);
        events.push(EcologicalEvent::Disease);
    } else {
        state.disease_pressure *= 0.92;
    }

    if rng.chance(environment.temperature_anomaly_probability) {
        state.temperature += rng.centered(environment.temperature_anomaly_width);
        state.temperature = state.temperature.max(0.0);
        events.push(EcologicalEvent::TemperatureAnomaly);
    } else {
        state.temperature = (state.temperature * 0.98) + (environment.baseline_temperature * 0.02);
    }

    if rng.chance(environment.habitat_disturbance_probability) {
        state.vegetation *= 1.0 - environment.habitat_disturbance_loss.clamp(0.0, 1.0);
        events.push(EcologicalEvent::HabitatDisturbance);
    }

    state.clamp_nonnegative();
    events
}

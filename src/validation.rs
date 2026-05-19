use std::fmt;

use crate::partition::PartitionMap;
use crate::patch::{Patch, PatchId};
use crate::worker::MigrationEvent;

#[derive(Clone, Debug)]
pub enum ValidationError {
    NegativePopulation {
        patch_id: PatchId,
        variable: &'static str,
        value: f64,
    },
    NonFiniteValue {
        patch_id: PatchId,
        variable: &'static str,
    },
    VegetationCapacityExceeded {
        patch_id: PatchId,
        value: f64,
        maximum: f64,
    },
    MigrationConservationFailure {
        variable: &'static str,
        expected: f64,
        actual: f64,
    },
    PatchCountChanged {
        expected: usize,
        actual: usize,
    },
    PatchOwnershipMissing {
        patch_id: PatchId,
    },
    PatchOwnershipDuplicated {
        patch_id: PatchId,
    },
    InvalidMigrationEvent {
        reason: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativePopulation {
                patch_id,
                variable,
                value,
            } => write!(
                formatter,
                "patch {} has negative {variable}: {value}",
                patch_id.0
            ),
            Self::NonFiniteValue { patch_id, variable } => {
                write!(formatter, "patch {} has non-finite {variable}", patch_id.0)
            }
            Self::VegetationCapacityExceeded {
                patch_id,
                value,
                maximum,
            } => write!(
                formatter,
                "patch {} vegetation {value} exceeded maximum {maximum}",
                patch_id.0
            ),
            Self::MigrationConservationFailure {
                variable,
                expected,
                actual,
            } => write!(
                formatter,
                "migration failed to conserve {variable}: expected {expected}, got {actual}"
            ),
            Self::PatchCountChanged { expected, actual } => {
                write!(formatter, "patch count changed from {expected} to {actual}")
            }
            Self::PatchOwnershipMissing { patch_id } => {
                write!(formatter, "patch {} has no partition owner", patch_id.0)
            }
            Self::PatchOwnershipDuplicated { patch_id } => {
                write!(
                    formatter,
                    "patch {} is owned by multiple partitions",
                    patch_id.0
                )
            }
            Self::InvalidMigrationEvent { reason } => {
                write!(formatter, "invalid migration event: {reason}")
            }
        }
    }
}

pub fn validate_patches(
    patches: &[Patch],
    expected_count: usize,
    vegetation_maximum: f64,
) -> Result<(), ValidationError> {
    if patches.len() != expected_count {
        return Err(ValidationError::PatchCountChanged {
            expected: expected_count,
            actual: patches.len(),
        });
    }

    for patch in patches {
        validate_value(patch, "prey", patch.state.prey)?;
        validate_value(patch, "predators", patch.state.predators)?;
        validate_value(patch, "vegetation", patch.state.vegetation)?;
        validate_value(patch, "rainfall", patch.state.rainfall)?;
        validate_value(patch, "temperature", patch.state.temperature)?;
        validate_value(patch, "disease_pressure", patch.state.disease_pressure)?;

        if patch.state.vegetation > vegetation_maximum + 1.0e-6 {
            return Err(ValidationError::VegetationCapacityExceeded {
                patch_id: patch.id,
                value: patch.state.vegetation,
                maximum: vegetation_maximum,
            });
        }
    }

    Ok(())
}

pub fn validate_migration(
    prey_before: f64,
    prey_after: f64,
    predators_before: f64,
    predators_after: f64,
    tolerance: f64,
) -> Result<(), ValidationError> {
    if (prey_before - prey_after).abs() > tolerance {
        return Err(ValidationError::MigrationConservationFailure {
            variable: "prey",
            expected: prey_before,
            actual: prey_after,
        });
    }

    if (predators_before - predators_after).abs() > tolerance {
        return Err(ValidationError::MigrationConservationFailure {
            variable: "predators",
            expected: predators_before,
            actual: predators_after,
        });
    }

    Ok(())
}

pub fn validate_partition_map(partition_map: &PartitionMap) -> Result<(), ValidationError> {
    let mut seen = vec![false; partition_map.patch_count()];

    for partition in partition_map.partitions() {
        for patch_id in &partition.patch_ids {
            let owner =
                partition_map
                    .owner(*patch_id)
                    .ok_or(ValidationError::PatchOwnershipMissing {
                        patch_id: *patch_id,
                    })?;

            if owner != partition.id {
                return Err(ValidationError::PatchOwnershipMissing {
                    patch_id: *patch_id,
                });
            }

            if seen[patch_id.0] {
                return Err(ValidationError::PatchOwnershipDuplicated {
                    patch_id: *patch_id,
                });
            }
            seen[patch_id.0] = true;
        }
    }

    for (patch_id, found) in seen.into_iter().enumerate() {
        if !found {
            return Err(ValidationError::PatchOwnershipMissing {
                patch_id: PatchId(patch_id),
            });
        }
    }

    Ok(())
}

pub fn validate_migration_events(
    events: &[MigrationEvent],
    expected_timestep: usize,
    patch_count: usize,
) -> Result<(), ValidationError> {
    for event in events {
        if event.timestep != expected_timestep {
            return Err(ValidationError::InvalidMigrationEvent {
                reason: format!(
                    "expected timestep {}, got {}",
                    expected_timestep, event.timestep
                ),
            });
        }

        if event.source_patch.0 >= patch_count {
            return Err(ValidationError::InvalidMigrationEvent {
                reason: format!("source patch {} is out of range", event.source_patch.0),
            });
        }

        if event.destination_patch.0 >= patch_count {
            return Err(ValidationError::InvalidMigrationEvent {
                reason: format!(
                    "destination patch {} is out of range",
                    event.destination_patch.0
                ),
            });
        }

        if !event.amount.is_finite() || event.amount < -1.0e-12 {
            return Err(ValidationError::InvalidMigrationEvent {
                reason: format!("event amount {} is not a valid migrant count", event.amount),
            });
        }
    }

    Ok(())
}

fn validate_value(
    patch: &Patch,
    variable: &'static str,
    value: f64,
) -> Result<(), ValidationError> {
    if !value.is_finite() {
        return Err(ValidationError::NonFiniteValue {
            patch_id: patch.id,
            variable,
        });
    }

    if value < -1.0e-9 {
        return Err(ValidationError::NegativePopulation {
            patch_id: patch.id,
            variable,
            value,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_migration_events, validate_partition_map};
    use crate::partition::PartitionMap;
    use crate::patch::PatchId;
    use crate::worker::{MigratingSpecies, MigrationEvent};

    #[test]
    fn validates_partition_ownership() {
        let map = PartitionMap::contiguous(9, 4);
        validate_partition_map(&map).expect("partition map should be valid");
    }

    #[test]
    fn rejects_future_migration_event() {
        let event = MigrationEvent {
            timestep: 3,
            source_patch: PatchId(0),
            destination_patch: PatchId(1),
            species: MigratingSpecies::Prey,
            amount: 1.0,
        };

        assert!(validate_migration_events(&[event], 2, 4).is_err());
    }
}

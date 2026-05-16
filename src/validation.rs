use std::fmt;

use crate::patch::{Patch, PatchId};

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

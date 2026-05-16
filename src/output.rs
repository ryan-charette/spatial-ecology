use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::OutputConfig;
use crate::metrics::SimulationSummary;
use crate::patch::Patch;

pub struct OutputWriters {
    results: BufWriter<File>,
    summary: BufWriter<File>,
}

impl OutputWriters {
    pub fn create(output: &OutputConfig) -> Result<Self, String> {
        ensure_parent(&output.results_csv)?;
        ensure_parent(&output.summary_csv)?;

        let results = File::create(&output.results_csv)
            .map_err(|error| format!("failed to create {}: {error}", output.results_csv))?;
        let summary = File::create(&output.summary_csv)
            .map_err(|error| format!("failed to create {}: {error}", output.summary_csv))?;

        let mut writers = Self {
            results: BufWriter::new(results),
            summary: BufWriter::new(summary),
        };

        writeln!(
            writers.results,
            "trial,timestep,patch_id,row,col,prey,predators,vegetation,rainfall,temperature,event,scenario,seed,disease_pressure"
        )
        .map_err(|error| format!("failed to write results header: {error}"))?;

        writeln!(
            writers.summary,
            "trial,seed,steps,rows,cols,final_prey,final_predators,final_vegetation,prey_extinct,predator_extinct,time_to_prey_extinction,time_to_predator_extinction,mean_prey,mean_predators,mean_vegetation,migration_rate,drought_probability,disease_probability,fragmentation_rate,predation_rate,stability_score,recovery_time_after_drought,scenario"
        )
        .map_err(|error| format!("failed to write summary header: {error}"))?;

        Ok(writers)
    }

    pub fn write_patch_record(
        &mut self,
        scenario: &str,
        trial: usize,
        seed: u64,
        timestep: usize,
        patch: &Patch,
        event: &str,
    ) -> Result<(), String> {
        writeln!(
            self.results,
            "{trial},{timestep},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{},{:.6}",
            patch.id.0,
            patch.coord.row,
            patch.coord.col,
            patch.state.prey,
            patch.state.predators,
            patch.state.vegetation,
            patch.state.rainfall,
            patch.state.temperature,
            csv_escape(event),
            csv_escape(scenario),
            seed,
            patch.state.disease_pressure
        )
        .map_err(|error| format!("failed to write patch record: {error}"))
    }

    pub fn write_summary(&mut self, summary: &SimulationSummary) -> Result<(), String> {
        writeln!(
            self.summary,
            "{},{},{},{},{},{:.6},{:.6},{:.6},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.8},{:.6},{},{}",
            summary.trial,
            summary.seed,
            summary.steps,
            summary.rows,
            summary.cols,
            summary.final_prey,
            summary.final_predators,
            summary.final_vegetation,
            summary.prey_extinct,
            summary.predator_extinct,
            optional_usize(summary.time_to_prey_extinction),
            optional_usize(summary.time_to_predator_extinction),
            summary.mean_prey,
            summary.mean_predators,
            summary.mean_vegetation,
            summary.migration_rate,
            summary.drought_probability,
            summary.disease_probability,
            summary.fragmentation_rate,
            summary.predation_rate,
            summary.stability_score,
            optional_usize(summary.recovery_time_after_drought),
            csv_escape(&summary.scenario)
        )
        .map_err(|error| format!("failed to write summary record: {error}"))
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.results
            .flush()
            .map_err(|error| format!("failed to flush results: {error}"))?;
        self.summary
            .flush()
            .map_err(|error| format!("failed to flush summary: {error}"))
    }
}

fn ensure_parent(path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }

    Ok(())
}

fn optional_usize(value: Option<usize>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::OutputConfig;
use crate::metrics::{SimulationSummary, TimestepMetricsRecord, WorkerMetricsRecord};
use crate::patch::Patch;

pub struct OutputWriters {
    results: BufWriter<File>,
    summary: BufWriter<File>,
    timestep_metrics: BufWriter<File>,
    worker_metrics: BufWriter<File>,
    benchmark: BufWriter<File>,
}

impl OutputWriters {
    pub fn create(output: &OutputConfig, append_benchmark: bool) -> Result<Self, String> {
        ensure_parent(&output.results_csv)?;
        ensure_parent(&output.summary_csv)?;
        ensure_parent(&output.timestep_metrics_csv)?;
        ensure_parent(&output.worker_metrics_csv)?;
        ensure_parent(&output.benchmark_csv)?;

        let results = File::create(&output.results_csv)
            .map_err(|error| format!("failed to create {}: {error}", output.results_csv))?;
        let summary = File::create(&output.summary_csv)
            .map_err(|error| format!("failed to create {}: {error}", output.summary_csv))?;
        let timestep_metrics = File::create(&output.timestep_metrics_csv).map_err(|error| {
            format!("failed to create {}: {error}", output.timestep_metrics_csv)
        })?;
        let worker_metrics = File::create(&output.worker_metrics_csv)
            .map_err(|error| format!("failed to create {}: {error}", output.worker_metrics_csv))?;
        let write_benchmark_header = !append_benchmark
            || !Path::new(&output.benchmark_csv).exists()
            || fs::metadata(&output.benchmark_csv)
                .map(|metadata| metadata.len() == 0)
                .unwrap_or(true);
        let benchmark = if append_benchmark {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&output.benchmark_csv)
        } else {
            File::create(&output.benchmark_csv)
        }
        .map_err(|error| format!("failed to create {}: {error}", output.benchmark_csv))?;

        let mut writers = Self {
            results: BufWriter::new(results),
            summary: BufWriter::new(summary),
            timestep_metrics: BufWriter::new(timestep_metrics),
            worker_metrics: BufWriter::new(worker_metrics),
            benchmark: BufWriter::new(benchmark),
        };

        writeln!(
            writers.results,
            "trial,timestep,patch_id,row,col,prey,predators,vegetation,rainfall,temperature,event,scenario,seed,disease_pressure"
        )
        .map_err(|error| format!("failed to write results header: {error}"))?;

        writeln!(
            writers.summary,
            "trial,seed,steps,rows,cols,final_prey,final_predators,final_vegetation,prey_extinct,predator_extinct,time_to_prey_extinction,time_to_predator_extinction,mean_prey,mean_predators,mean_vegetation,migration_rate,drought_probability,disease_probability,fragmentation_rate,predation_rate,stability_score,recovery_time_after_drought,scenario,execution_mode,workers,total_runtime_ms,mean_timestep_ms,total_events,cross_worker_events,total_edges,local_edges,boundary_edges,boundary_edge_fraction,patches_per_second,events_per_second"
        )
        .map_err(|error| format!("failed to write summary header: {error}"))?;

        writeln!(
            writers.timestep_metrics,
            "scenario,trial,timestep,execution_mode,workers,patches,total_events,cross_worker_events,timestep_ms,worker_compute_ms,event_exchange_ms,migration_apply_ms,environment_ms,metrics_aggregation_ms,validation_ms,total_prey,total_predators,total_vegetation"
        )
        .map_err(|error| format!("failed to write timestep metrics header: {error}"))?;

        writeln!(
            writers.worker_metrics,
            "scenario,trial,timestep,worker_id,owned_patches,outgoing_events,incoming_events,local_update_ms,event_generation_ms,mean_prey,mean_predators,mean_vegetation"
        )
        .map_err(|error| format!("failed to write worker metrics header: {error}"))?;

        if write_benchmark_header {
            writeln!(
            writers.benchmark,
            "scenario,trial,execution_mode,workers,steps,patches,total_edges,boundary_edges,total_runtime_ms,mean_timestep_ms,patches_per_second,events_per_second,total_events,cross_worker_events,boundary_edge_fraction,speedup_vs_serial,parallel_efficiency"
        )
            .map_err(|error| format!("failed to write benchmark header: {error}"))?;
        }

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
            "{},{},{},{},{},{:.6},{:.6},{:.6},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.8},{:.6},{},{},{},{},{:.6},{:.6},{},{},{},{},{},{:.6},{:.6},{:.6}",
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
            csv_escape(&summary.scenario),
            summary.systems.mode.as_str(),
            summary.systems.workers,
            summary.systems.total_runtime_ms,
            summary.systems.mean_timestep_ms,
            summary.systems.total_events,
            summary.systems.cross_worker_events,
            summary.systems.total_edges,
            summary.systems.local_edges,
            summary.systems.boundary_edges,
            summary.systems.boundary_edge_fraction,
            summary.systems.patches_per_second,
            summary.systems.events_per_second
        )
        .map_err(|error| format!("failed to write summary record: {error}"))
    }

    pub fn write_timestep_metrics(&mut self, record: &TimestepMetricsRecord) -> Result<(), String> {
        writeln!(
            self.timestep_metrics,
            "{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            csv_escape(&record.scenario),
            record.trial,
            record.timestep,
            record.mode.as_str(),
            record.workers,
            record.patches,
            record.total_events,
            record.cross_worker_events,
            record.timestep_ms,
            record.worker_compute_ms,
            record.event_exchange_ms,
            record.migration_apply_ms,
            record.environment_ms,
            record.metrics_aggregation_ms,
            record.validation_ms,
            record.total_prey,
            record.total_predators,
            record.total_vegetation
        )
        .map_err(|error| format!("failed to write timestep metrics record: {error}"))
    }

    pub fn write_worker_metrics(&mut self, record: &WorkerMetricsRecord) -> Result<(), String> {
        writeln!(
            self.worker_metrics,
            "{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6}",
            csv_escape(&record.scenario),
            record.trial,
            record.timestep,
            record.worker_id,
            record.owned_patches,
            record.outgoing_events,
            record.incoming_events,
            record.local_update_ms,
            record.event_generation_ms,
            record.mean_prey,
            record.mean_predators,
            record.mean_vegetation
        )
        .map_err(|error| format!("failed to write worker metrics record: {error}"))
    }

    pub fn write_benchmark_record(
        &mut self,
        summary: &SimulationSummary,
        serial_runtime_ms: Option<f64>,
    ) -> Result<(), String> {
        let patches = summary.rows * summary.cols;
        let speedup = serial_runtime_ms
            .filter(|runtime| *runtime > 0.0 && summary.systems.total_runtime_ms > 0.0)
            .map(|runtime| runtime / summary.systems.total_runtime_ms);
        let efficiency = speedup.map(|value| value / summary.systems.workers.max(1) as f64);

        writeln!(
            self.benchmark,
            "{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{},{:.6},{},{}",
            csv_escape(&summary.scenario),
            summary.trial,
            summary.systems.mode.as_str(),
            summary.systems.workers,
            summary.steps,
            patches,
            summary.systems.total_edges,
            summary.systems.boundary_edges,
            summary.systems.total_runtime_ms,
            summary.systems.mean_timestep_ms,
            summary.systems.patches_per_second,
            summary.systems.events_per_second,
            summary.systems.total_events,
            summary.systems.cross_worker_events,
            summary.systems.boundary_edge_fraction,
            optional_f64(speedup),
            optional_f64(efficiency)
        )
        .map_err(|error| format!("failed to write benchmark record: {error}"))
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.results
            .flush()
            .map_err(|error| format!("failed to flush results: {error}"))?;
        self.summary
            .flush()
            .map_err(|error| format!("failed to flush summary: {error}"))?;
        self.timestep_metrics
            .flush()
            .map_err(|error| format!("failed to flush timestep metrics: {error}"))?;
        self.worker_metrics
            .flush()
            .map_err(|error| format!("failed to flush worker metrics: {error}"))?;
        self.benchmark
            .flush()
            .map_err(|error| format!("failed to flush benchmark metrics: {error}"))
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

fn optional_f64(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.6}")).unwrap_or_default()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub scenario_name: String,
    pub simulation: SimulationSection,
    pub execution: ExecutionConfig,
    pub initial_conditions: InitialConditions,
    pub biology: BiologyConfig,
    pub migration: MigrationConfig,
    pub environment: EnvironmentConfig,
    pub thresholds: ThresholdConfig,
    pub output: OutputConfig,
    pub sweep: SweepConfig,
}

#[derive(Clone, Debug)]
pub struct SimulationSection {
    pub steps: usize,
    pub seed: u64,
    pub trials: usize,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub workers: usize,
    pub benchmark: bool,
    pub validate: bool,
}

#[derive(Clone, Debug)]
pub struct RuntimeOptions {
    pub mode: ExecutionMode,
    pub workers: usize,
    pub benchmark: bool,
    pub validate: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Serial,
    Parallel,
}

impl ExecutionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serial => "serial",
            Self::Parallel => "parallel",
        }
    }
}

#[derive(Clone, Debug)]
pub struct InitialConditions {
    pub prey: f64,
    pub predators: f64,
    pub vegetation: f64,
    pub rainfall: f64,
    pub temperature: f64,
    pub disease_pressure: f64,
}

#[derive(Clone, Debug)]
pub struct BiologyConfig {
    pub vegetation_growth_rate: f64,
    pub vegetation_grazing_rate: f64,
    pub prey_birth_rate: f64,
    pub prey_death_rate: f64,
    pub predation_rate: f64,
    pub predator_conversion_efficiency: f64,
    pub predator_death_rate: f64,
    pub carrying_capacity: f64,
    pub optimal_temperature: f64,
    pub temperature_tolerance: f64,
}

#[derive(Clone, Debug)]
pub struct MigrationConfig {
    pub prey_migration_rate: f64,
    pub predator_migration_rate: f64,
    pub fragmentation_rate: f64,
    pub scarcity_threshold: f64,
    pub scarcity_migration_multiplier: f64,
}

#[derive(Clone, Debug)]
pub struct EnvironmentConfig {
    pub drought_probability: f64,
    pub drought_vegetation_loss: f64,
    pub disease_probability: f64,
    pub disease_prey_mortality: f64,
    pub temperature_anomaly_probability: f64,
    pub temperature_anomaly_width: f64,
    pub baseline_temperature: f64,
    pub habitat_disturbance_probability: f64,
    pub habitat_disturbance_loss: f64,
}

#[derive(Clone, Debug)]
pub struct ThresholdConfig {
    pub prey_extinction_threshold: f64,
    pub predator_extinction_threshold: f64,
}

#[derive(Clone, Debug)]
pub struct OutputConfig {
    pub results_csv: String,
    pub summary_csv: String,
    pub timestep_metrics_csv: String,
    pub worker_metrics_csv: String,
    pub benchmark_csv: String,
}

#[derive(Clone, Debug, Default)]
pub struct SweepConfig {
    pub migration_rates: Vec<f64>,
    pub drought_probabilities: Vec<f64>,
    pub fragmentation_rates: Vec<f64>,
    pub predation_rates: Vec<f64>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            scenario_name: String::from("baseline"),
            simulation: SimulationSection::default(),
            execution: ExecutionConfig::default(),
            initial_conditions: InitialConditions::default(),
            biology: BiologyConfig::default(),
            migration: MigrationConfig::default(),
            environment: EnvironmentConfig::default(),
            thresholds: ThresholdConfig::default(),
            output: OutputConfig::default(),
            sweep: SweepConfig::default(),
        }
    }
}

impl Default for SimulationSection {
    fn default() -> Self {
        Self {
            steps: 500,
            seed: 42,
            trials: 1,
            rows: 10,
            cols: 10,
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Serial,
            workers: 1,
            benchmark: false,
            validate: true,
        }
    }
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            prey: 100.0,
            predators: 20.0,
            vegetation: 500.0,
            rainfall: 1.0,
            temperature: 20.0,
            disease_pressure: 0.0,
        }
    }
}

impl Default for BiologyConfig {
    fn default() -> Self {
        Self {
            vegetation_growth_rate: 0.05,
            vegetation_grazing_rate: 0.03,
            prey_birth_rate: 0.04,
            prey_death_rate: 0.01,
            predation_rate: 0.001,
            predator_conversion_efficiency: 0.10,
            predator_death_rate: 0.02,
            carrying_capacity: 1000.0,
            optimal_temperature: 20.0,
            temperature_tolerance: 18.0,
        }
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            prey_migration_rate: 0.02,
            predator_migration_rate: 0.01,
            fragmentation_rate: 0.10,
            scarcity_threshold: 0.25,
            scarcity_migration_multiplier: 1.5,
        }
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            drought_probability: 0.02,
            drought_vegetation_loss: 0.25,
            disease_probability: 0.01,
            disease_prey_mortality: 0.15,
            temperature_anomaly_probability: 0.005,
            temperature_anomaly_width: 6.0,
            baseline_temperature: 20.0,
            habitat_disturbance_probability: 0.002,
            habitat_disturbance_loss: 0.20,
        }
    }
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            prey_extinction_threshold: 1.0,
            predator_extinction_threshold: 1.0,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            results_csv: String::from("results/baseline.csv"),
            summary_csv: String::from("results/summary.csv"),
            timestep_metrics_csv: String::from("results/timestep_metrics.csv"),
            worker_metrics_csv: String::from("results/worker_metrics.csv"),
            benchmark_csv: String::from("results/benchmarks/scaling.csv"),
        }
    }
}

impl OutputConfig {
    pub fn with_directory(mut self, target: &str) -> Self {
        if target.to_ascii_lowercase().ends_with(".csv") {
            let path = Path::new(target);
            let base = path
                .parent()
                .and_then(|parent| parent.to_str())
                .filter(|parent| !parent.is_empty())
                .unwrap_or("results");
            self.results_csv = target.to_string();
            self.summary_csv = format!("{base}/summary.csv");
            self.timestep_metrics_csv = format!("{base}/timestep_metrics.csv");
            self.worker_metrics_csv = format!("{base}/worker_metrics.csv");
            self.benchmark_csv = format!("{base}/benchmarks/scaling.csv");
            return self;
        }

        let base = target.trim_end_matches(&['/', '\\'][..]);
        self.results_csv = format!("{base}/baseline.csv");
        self.summary_csv = format!("{base}/summary.csv");
        self.timestep_metrics_csv = format!("{base}/timestep_metrics.csv");
        self.worker_metrics_csv = format!("{base}/worker_metrics.csv");
        self.benchmark_csv = format!("{base}/benchmarks/scaling.csv");
        self
    }
}

impl RuntimeOptions {
    pub fn from_config(config: &ExecutionConfig) -> Self {
        Self {
            mode: config.mode,
            workers: config.workers,
            benchmark: config.benchmark,
            validate: config.validate,
        }
    }

    pub fn effective_workers(&self) -> usize {
        match self.mode {
            ExecutionMode::Serial => 1,
            ExecutionMode::Parallel => self.workers.max(1),
        }
    }
}

impl SimulationConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents =
            fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
        Self::from_toml_str(&contents)
    }

    pub fn from_toml_str(contents: &str) -> Result<Self, String> {
        let mut config = Self::default();
        let mut section = String::new();

        for (line_number, raw_line) in contents.lines().enumerate() {
            let line = strip_comment(raw_line).trim().to_string();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid config line {}: {raw_line}", line_number + 1))?;
            config.set_value(section.trim(), key.trim(), value.trim())?;
        }

        config.validate()?;
        Ok(config)
    }

    pub fn scenario_configs(&self) -> Vec<Self> {
        let has_sweep = !self.sweep.migration_rates.is_empty()
            || !self.sweep.drought_probabilities.is_empty()
            || !self.sweep.fragmentation_rates.is_empty()
            || !self.sweep.predation_rates.is_empty();
        if !has_sweep {
            return vec![self.clone()];
        }

        let migration_rates = values_or_default(
            &self.sweep.migration_rates,
            self.migration.prey_migration_rate,
        );
        let drought_probabilities = values_or_default(
            &self.sweep.drought_probabilities,
            self.environment.drought_probability,
        );
        let fragmentation_rates = values_or_default(
            &self.sweep.fragmentation_rates,
            self.migration.fragmentation_rate,
        );
        let predation_rates =
            values_or_default(&self.sweep.predation_rates, self.biology.predation_rate);

        let predator_ratio = if self.migration.prey_migration_rate > 0.0 {
            self.migration.predator_migration_rate / self.migration.prey_migration_rate
        } else {
            0.5
        };

        let mut scenarios = Vec::new();
        for migration_rate in migration_rates {
            for drought_probability in &drought_probabilities {
                for fragmentation_rate in &fragmentation_rates {
                    for predation_rate in &predation_rates {
                        let mut scenario = self.clone();
                        scenario.migration.prey_migration_rate = migration_rate;
                        scenario.migration.predator_migration_rate =
                            (migration_rate * predator_ratio).clamp(0.0, 1.0);
                        scenario.environment.drought_probability = *drought_probability;
                        scenario.migration.fragmentation_rate = *fragmentation_rate;
                        scenario.biology.predation_rate = *predation_rate;
                        scenario.scenario_name = format!(
                            "migration_{:.3}_drought_{:.3}_fragmentation_{:.3}_predation_{:.5}",
                            migration_rate, drought_probability, fragmentation_rate, predation_rate
                        );
                        scenarios.push(scenario);
                    }
                }
            }
        }

        scenarios
    }

    fn set_value(&mut self, section: &str, key: &str, value: &str) -> Result<(), String> {
        match (section, key) {
            ("project", "scenario_name") => self.scenario_name = parse_string(value),
            ("simulation", "steps") => self.simulation.steps = parse_usize(key, value)?,
            ("simulation", "seed") => self.simulation.seed = parse_u64(key, value)?,
            ("simulation", "trials") => self.simulation.trials = parse_usize(key, value)?,
            ("simulation", "rows") => self.simulation.rows = parse_usize(key, value)?,
            ("simulation", "cols") => self.simulation.cols = parse_usize(key, value)?,
            ("execution", "mode") => self.execution.mode = parse_execution_mode(value)?,
            ("execution", "workers") => self.execution.workers = parse_usize(key, value)?,
            ("execution", "benchmark") => self.execution.benchmark = parse_bool(key, value)?,
            ("execution", "validate") => self.execution.validate = parse_bool(key, value)?,
            ("initial_conditions", "prey") => self.initial_conditions.prey = parse_f64(key, value)?,
            ("initial_conditions", "predators") => {
                self.initial_conditions.predators = parse_f64(key, value)?
            }
            ("initial_conditions", "vegetation") => {
                self.initial_conditions.vegetation = parse_f64(key, value)?
            }
            ("initial_conditions", "rainfall") => {
                self.initial_conditions.rainfall = parse_f64(key, value)?
            }
            ("initial_conditions", "temperature") => {
                self.initial_conditions.temperature = parse_f64(key, value)?
            }
            ("initial_conditions", "disease_pressure") => {
                self.initial_conditions.disease_pressure = parse_f64(key, value)?
            }
            ("biology", "vegetation_growth_rate") => {
                self.biology.vegetation_growth_rate = parse_f64(key, value)?
            }
            ("biology", "vegetation_grazing_rate") => {
                self.biology.vegetation_grazing_rate = parse_f64(key, value)?
            }
            ("biology", "prey_birth_rate") => self.biology.prey_birth_rate = parse_f64(key, value)?,
            ("biology", "prey_death_rate") => self.biology.prey_death_rate = parse_f64(key, value)?,
            ("biology", "predation_rate") => self.biology.predation_rate = parse_f64(key, value)?,
            ("biology", "predator_conversion_efficiency") => {
                self.biology.predator_conversion_efficiency = parse_f64(key, value)?
            }
            ("biology", "predator_death_rate") => {
                self.biology.predator_death_rate = parse_f64(key, value)?
            }
            ("biology", "carrying_capacity") => {
                self.biology.carrying_capacity = parse_f64(key, value)?
            }
            ("biology", "optimal_temperature") => {
                self.biology.optimal_temperature = parse_f64(key, value)?
            }
            ("biology", "temperature_tolerance") => {
                self.biology.temperature_tolerance = parse_f64(key, value)?
            }
            ("migration", "prey_migration_rate") => {
                self.migration.prey_migration_rate = parse_f64(key, value)?
            }
            ("migration", "predator_migration_rate") => {
                self.migration.predator_migration_rate = parse_f64(key, value)?
            }
            ("migration", "fragmentation_rate") => {
                self.migration.fragmentation_rate = parse_f64(key, value)?
            }
            ("migration", "scarcity_threshold") => {
                self.migration.scarcity_threshold = parse_f64(key, value)?
            }
            ("migration", "scarcity_migration_multiplier") => {
                self.migration.scarcity_migration_multiplier = parse_f64(key, value)?
            }
            ("environment", "drought_probability") => {
                self.environment.drought_probability = parse_f64(key, value)?
            }
            ("environment", "drought_vegetation_loss") => {
                self.environment.drought_vegetation_loss = parse_f64(key, value)?
            }
            ("environment", "disease_probability") => {
                self.environment.disease_probability = parse_f64(key, value)?
            }
            ("environment", "disease_prey_mortality") => {
                self.environment.disease_prey_mortality = parse_f64(key, value)?
            }
            ("environment", "temperature_anomaly_probability") => {
                self.environment.temperature_anomaly_probability = parse_f64(key, value)?
            }
            ("environment", "temperature_anomaly_width") => {
                self.environment.temperature_anomaly_width = parse_f64(key, value)?
            }
            ("environment", "baseline_temperature") => {
                self.environment.baseline_temperature = parse_f64(key, value)?
            }
            ("environment", "habitat_disturbance_probability") => {
                self.environment.habitat_disturbance_probability = parse_f64(key, value)?
            }
            ("environment", "habitat_disturbance_loss") => {
                self.environment.habitat_disturbance_loss = parse_f64(key, value)?
            }
            ("thresholds", "prey_extinction_threshold") => {
                self.thresholds.prey_extinction_threshold = parse_f64(key, value)?
            }
            ("thresholds", "predator_extinction_threshold") => {
                self.thresholds.predator_extinction_threshold = parse_f64(key, value)?
            }
            ("output", "results_csv") => self.output.results_csv = parse_string(value),
            ("output", "summary_csv") => self.output.summary_csv = parse_string(value),
            ("output", "timestep_metrics_csv") => {
                self.output.timestep_metrics_csv = parse_string(value)
            }
            ("output", "worker_metrics_csv") => {
                self.output.worker_metrics_csv = parse_string(value)
            }
            ("output", "benchmark_csv") => self.output.benchmark_csv = parse_string(value),
            ("sweep", "migration_rates") => self.sweep.migration_rates = parse_f64_array(value)?,
            ("sweep", "drought_probabilities") => {
                self.sweep.drought_probabilities = parse_f64_array(value)?
            }
            ("sweep", "fragmentation_rates") => {
                self.sweep.fragmentation_rates = parse_f64_array(value)?
            }
            ("sweep", "predation_rates") => self.sweep.predation_rates = parse_f64_array(value)?,
            _ => return Err(format!("unknown config field [{section}] {key}")),
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.simulation.rows == 0 || self.simulation.cols == 0 {
            return Err(String::from("rows and cols must both be positive"));
        }

        if self.simulation.trials == 0 {
            return Err(String::from("trials must be positive"));
        }

        if self.execution.workers == 0 {
            return Err(String::from("workers must be positive"));
        }

        if self.biology.carrying_capacity <= 0.0 {
            return Err(String::from("carrying_capacity must be positive"));
        }

        if self.biology.temperature_tolerance <= 0.0 {
            return Err(String::from("temperature_tolerance must be positive"));
        }

        Ok(())
    }
}

fn values_or_default(values: &[f64], default: f64) -> Vec<f64> {
    if values.is_empty() {
        vec![default]
    } else {
        values.to_vec()
    }
}

fn strip_comment(line: &str) -> String {
    let mut in_quotes = false;
    let mut result = String::new();

    for character in line.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                result.push(character);
            }
            '#' if !in_quotes => break,
            _ => result.push(character),
        }
    }

    result
}

fn parse_string(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_usize(key: &str, value: &str) -> Result<usize, String> {
    value
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("{key} must be a positive integer"))
}

fn parse_u64(key: &str, value: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{key} must be a nonnegative integer"))
}

pub fn parse_execution_mode(value: &str) -> Result<ExecutionMode, String> {
    match parse_string(value).to_ascii_lowercase().as_str() {
        "serial" => Ok(ExecutionMode::Serial),
        "parallel" => Ok(ExecutionMode::Parallel),
        other => Err(format!("mode must be serial or parallel, got {other}")),
    }
}

fn parse_bool(key: &str, value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{key} must be true or false")),
    }
}

fn parse_f64(key: &str, value: &str) -> Result<f64, String> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("{key} must be a number"))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(format!("{key} must be finite"))
    }
}

fn parse_f64_array(value: &str) -> Result<Vec<f64>, String> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(format!("expected an array, got {value}"));
    }

    let inner = trimmed[1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    inner
        .split(',')
        .map(|item| parse_f64("array item", item.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ExecutionMode, SimulationConfig};

    #[test]
    fn parses_minimal_config() {
        let config = SimulationConfig::from_toml_str(
            r#"
            [simulation]
            steps = 12
            rows = 2
            cols = 3

            [output]
            results_csv = "results/test.csv"
            "#,
        )
        .expect("config should parse");

        assert_eq!(config.simulation.steps, 12);
        assert_eq!(config.simulation.rows * config.simulation.cols, 6);
        assert_eq!(config.output.results_csv, "results/test.csv");
    }

    #[test]
    fn parses_execution_config() {
        let config = SimulationConfig::from_toml_str(
            r#"
            [execution]
            mode = "parallel"
            workers = 4
            benchmark = true
            validate = true
            "#,
        )
        .expect("config should parse");

        assert_eq!(config.execution.mode, ExecutionMode::Parallel);
        assert_eq!(config.execution.workers, 4);
        assert!(config.execution.benchmark);
        assert!(config.execution.validate);
    }

    #[test]
    fn creates_sweep_scenarios() {
        let config = SimulationConfig::from_toml_str(
            r#"
            [sweep]
            migration_rates = [0.0, 0.02]
            drought_probabilities = [0.0, 0.1]
            "#,
        )
        .expect("config should parse");

        assert_eq!(config.scenario_configs().len(), 4);
    }
}

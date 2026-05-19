mod climate;
mod config;
mod event;
mod experiment;
mod metrics;
mod output;
mod parallel;
mod partition;
mod patch;
mod simulation;
mod species;
mod validation;
mod worker;

use std::env;

use config::{parse_execution_mode, ExecutionMode, RuntimeOptions, SimulationConfig};
use experiment::run_experiments;

#[derive(Debug)]
struct Cli {
    config_path: String,
    trials: Option<usize>,
    seed: Option<u64>,
    steps: Option<usize>,
    mode: Option<ExecutionMode>,
    workers: Option<usize>,
    benchmark: bool,
    validate: bool,
    output_directory: Option<String>,
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("parallel-spatial-ecosystems: {error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), String> {
    let cli = parse_cli(env::args().skip(1))?;
    let mut config = SimulationConfig::from_file(&cli.config_path)?;

    if let Some(trials) = cli.trials {
        config.simulation.trials = trials;
    }

    if let Some(seed) = cli.seed {
        config.simulation.seed = seed;
    }

    if let Some(steps) = cli.steps {
        config.simulation.steps = steps;
    }

    if let Some(output_directory) = cli.output_directory {
        config.output = config.output.clone().with_directory(&output_directory);
    }

    let mut runtime = RuntimeOptions::from_config(&config.execution);
    if let Some(mode) = cli.mode {
        runtime.mode = mode;
    }
    if let Some(workers) = cli.workers {
        runtime.workers = workers;
    }
    if cli.benchmark {
        runtime.benchmark = true;
    }
    if cli.validate {
        runtime.validate = true;
    }

    let validation_requested = cli.validate && runtime.validate;
    let report = run_experiments(config, runtime)?;
    println!(
        "Parallel Spatial Ecosystem Dynamics completed {} scenario(s) x {} trial(s) using {} worker(s) in {} mode. Results: {}. Summary: {}.",
        report.scenarios,
        report.trials_per_scenario,
        report.workers,
        report.mode.as_str(),
        report.results_csv,
        report.summary_csv
    );
    if validation_requested {
        println!(
            "Validation passed: patch ownership, migration conservation, timestep synchronization, and population bounds were checked."
        );
    }
    Ok(())
}

fn parse_cli<I>(mut args: I) -> Result<Cli, String>
where
    I: Iterator<Item = String>,
{
    let mut config_path = String::from("configs/baseline.toml");
    let mut trials = None;
    let mut seed = None;
    let mut steps = None;
    let mut mode = None;
    let mut workers = None;
    let mut benchmark = false;
    let mut validate = false;
    let mut output_directory = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-c" | "--config" | "--sweep" => {
                config_path = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a path"))?;
            }
            "-t" | "--trials" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--trials requires a value"))?;
                trials = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid trial count: {value}"))?,
                );
            }
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--seed requires a value"))?;
                seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid seed: {value}"))?,
                );
            }
            "--steps" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--steps requires a value"))?;
                steps = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid step count: {value}"))?,
                );
            }
            "--mode" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--mode requires serial or parallel"))?;
                mode = Some(parse_execution_mode(&value)?);
            }
            "--workers" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("--workers requires a value"))?;
                workers = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid worker count: {value}"))?,
                );
            }
            "--benchmark" => benchmark = true,
            "--validate" => validate = true,
            "--output" => {
                output_directory = Some(
                    args.next()
                        .ok_or_else(|| String::from("--output requires a path"))?,
                );
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    if let Some(0) = workers {
        return Err(String::from("--workers must be positive"));
    }

    Ok(Cli {
        config_path,
        trials,
        seed,
        steps,
        mode,
        workers,
        benchmark,
        validate,
        output_directory,
    })
}

fn print_help() {
    println!(
        "Parallel Spatial Ecosystem Dynamics\n\nUSAGE:\n    cargo run --release -- --config configs/baseline.toml --mode parallel --workers 4 --trials 10 --seed 42\n\nOPTIONS:\n    -c, --config <PATH>   TOML configuration file\n        --sweep <PATH>    Alias for --config, intended for sweep configs\n    -t, --trials <N>      Override the trial count in the config\n        --steps <N>       Override timestep count\n        --seed <N>        Override the base random seed\n        --mode <MODE>     serial or parallel\n        --workers <N>     Worker partitions for parallel execution\n        --benchmark       Write benchmark/scaling metrics\n        --validate        Enable extra partition and event validation\n        --output <PATH>   Directory for CSVs, or patch timeseries CSV path\n    -h, --help            Show this help"
    );
}

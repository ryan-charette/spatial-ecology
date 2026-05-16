mod climate;
mod config;
mod event;
mod experiment;
mod metrics;
mod output;
mod patch;
mod simulation;
mod species;
mod validation;

use std::env;

use config::SimulationConfig;
use experiment::run_experiments;

#[derive(Debug)]
struct Cli {
    config_path: String,
    trials: Option<usize>,
    seed: Option<u64>,
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("spatial-ecology-rs: {error}");
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

    let report = run_experiments(config)?;
    println!(
        "Spatial Ecology RS completed {} scenario(s) x {} trial(s). Results: {}. Summary: {}.",
        report.scenarios, report.trials_per_scenario, report.results_csv, report.summary_csv
    );
    Ok(())
}

fn parse_cli<I>(mut args: I) -> Result<Cli, String>
where
    I: Iterator<Item = String>,
{
    let mut config_path = String::from("configs/baseline.toml");
    let mut trials = None;
    let mut seed = None;

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
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    Ok(Cli {
        config_path,
        trials,
        seed,
    })
}

fn print_help() {
    println!(
        "Spatial Ecology RS\n\nUSAGE:\n    cargo run --release -- --config configs/baseline.toml --trials 10 --seed 42\n\nOPTIONS:\n    -c, --config <PATH>   TOML configuration file\n        --sweep <PATH>    Alias for --config, intended for sweep configs\n    -t, --trials <N>      Override the trial count in the config\n        --seed <N>        Override the base random seed\n    -h, --help            Show this help"
    );
}

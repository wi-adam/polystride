use polystride_lab::agent::{self, AgentConfig};
use polystride_lab::model::{Job, MAX_ARTIFACT, Recipe};
use polystride_lab::store::{CoordinatorConfig, Store};
use polystride_lab::{Result, error, read_bounded, read_json, transport};
use std::path::Path;
use std::process::ExitCode;

const HELP: &str = "\
Polystride Lab — resident pull agent and execution evidence

Usage:
  polystride-lab coordinator CONFIG.json
  polystride-lab agent CONFIG.json [--once]
  polystride-lab queue COORDINATOR.json RECIPE.json ARTIFACT
  polystride-lab status COORDINATOR.json

The operator queues locally; agents connect outbound over authenticated TLS.
Paths in configuration are relative to the process working directory.
Use dedicated test accounts and trusted single-process probes.
See docs/lab.md for setup, limits and the M0.5 acceptance gates.";

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => println!("{HELP}"),
        [arg] if arg == "--help" || arg == "-h" => println!("{HELP}"),
        [arg] if arg == "--version" || arg == "-V" => {
            println!("polystride-lab {}", env!("CARGO_PKG_VERSION"));
        }
        [command, config] if command == "coordinator" => {
            transport::serve(&read_json(Path::new(config))?)?;
        }
        [command, config] if command == "agent" => {
            let config: AgentConfig = read_json(Path::new(config))?;
            agent::run(&config, false)?;
        }
        [command, config, flag] if command == "agent" && flag == "--once" => {
            agent::run(&read_json(Path::new(config))?, true)?;
        }
        [command, config, recipe, artifact] if command == "queue" => {
            let config: CoordinatorConfig = read_json(Path::new(config))?;
            let recipe: Recipe = read_json(Path::new(recipe))?;
            let job = Job::new(recipe, read_bounded(Path::new(artifact), MAX_ARTIFACT)?)?;
            Store::open(&config.state_dir, config.hosts)?.queue(&job)?;
            println!(
                "queued {}: {} bytes, sha256 {}",
                job.recipe.id,
                job.artifact.len(),
                job.artifact_sha256
            );
        }
        [command, config] if command == "status" => {
            let config: CoordinatorConfig = read_json(Path::new(config))?;
            let statuses = Store::open(&config.state_dir, config.hosts)?.status()?;
            println!("{}", serde_json::to_string_pretty(&statuses)?);
        }
        _ => return Err(error("invalid arguments; run polystride-lab --help")),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("polystride-lab: {err}");
            ExitCode::FAILURE
        }
    }
}

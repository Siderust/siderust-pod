use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "siderust-pod",
    about = "Siderust POD command-line interface",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Validate a run configuration without executing it.
    ValidateConfig {
        /// Path to the YAML run configuration.
        config: String,
    },
    /// Run a configuration end-to-end.
    Run {
        /// Path to the YAML run configuration.
        config: String,
    },
    /// Pretty-print a previously written run manifest.
    InspectManifest {
        /// Path to `run.manifest.json`.
        manifest: String,
    },
    /// Pretty-print a previously written `qc.json`.
    Qc {
        /// Path to `qc.json`.
        qc: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    match cli.command {
        Cmd::ValidateConfig { config } => {
            let cfg = siderust_pod_service::RunConfig::from_yaml_file(&config)?;
            cfg.validate().map_err(|e| anyhow::anyhow!(e))?;
            println!("OK: {} validates", config);
        }
        Cmd::Run { config } => {
            let cfg = siderust_pod_service::RunConfig::from_yaml_file(&config)?;
            let report = siderust_pod_service::run(&cfg, &config)?;
            println!(
                "OK: ran {} steps, final epoch JD={}, manifest at {}",
                report.n_steps,
                report.final_state.epoch_tt.jd_value(),
                report.manifest_path.display()
            );
        }
        Cmd::InspectManifest { manifest } => {
            let txt = std::fs::read_to_string(&manifest)?;
            println!("{}", txt);
        }
        Cmd::Qc { qc } => {
            let txt = std::fs::read_to_string(&qc)?;
            println!("{}", txt);
        }
    }
    Ok(())
}

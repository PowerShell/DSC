// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    io::{self, IsTerminal, Read},
    process::exit,
};

use clap::Parser;
use rust_i18n::{i18n, t};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use args::{Arguments, SubCommand, TraceFormat, TraceLevel};
use discovery::RegistryClient;
use models::DiscoveryInput;

pub mod args;
pub mod discovery;
pub mod error;
pub mod models;

i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_DISCOVERY_ERROR: i32 = 2;

fn main() {
    let args = Arguments::parse();

    enable_tracing(&args.trace_level, &args.trace_format);

    match args.subcommand {
        SubCommand::Discover => run_discover(),
    }

    exit(EXIT_SUCCESS);
}

fn run_discover() {
    info!("{}", t!("main.startingDiscovery"));

    // Read discovery configuration from STDIN (if data is piped), falling back
    // to an empty config that may be supplemented via environment variables.
    let input = read_stdin_input();

    let registries = if input.registries.is_empty() {
        // Fall back to DSC_ORAS_REGISTRIES environment variable.
        let from_env = std::env::var("DSC_ORAS_REGISTRIES")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| models::RegistryReference { reference: s.trim().to_string() })
            .collect::<Vec<_>>();

        if from_env.is_empty() {
            warn!("{}", t!("main.noRegistries"));
            return;
        }
        from_env
    } else {
        input.registries.clone()
    };

    let effective_input = DiscoveryInput {
        registries: registries.clone(),
        kind: input.kind.clone(),
        version: input.version.clone(),
        platform: input.platform.clone(),
    };

    for registry_ref in &registries {
        debug!("Querying registry: {}", registry_ref.reference);

        let mut client = match RegistryClient::new(&registry_ref.reference) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to initialise client for '{}': {e}", registry_ref.reference);
                exit(EXIT_INVALID_ARGS);
            }
        };

        match client.discover(&effective_input) {
            Ok(artifacts) => {
                if artifacts.is_empty() {
                    debug!(
                        "No matching artifacts found in '{}'",
                        registry_ref.reference
                    );
                }
                for artifact in artifacts {
                    match serde_json::to_string(&artifact) {
                        Ok(json) => println!("{json}"),
                        Err(e) => {
                            error!("Failed to serialise discovered artifact: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "Discovery failed for '{}': {e}",
                    registry_ref.reference
                );
                exit(EXIT_DISCOVERY_ERROR);
            }
        }
    }

    info!("{}", t!("discovery.discoveryComplete"));
}

/// Reads a [`DiscoveryInput`] from STDIN when data is available, otherwise
/// returns a default (empty) configuration.
fn read_stdin_input() -> DiscoveryInput {
    if std::io::stdin().is_terminal() {
        debug!("{}", t!("main.emptyStdin"));
        return DiscoveryInput::default();
    }

    debug!("{}", t!("main.readStdin"));

    let mut buffer: Vec<u8> = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut buffer) {
        error!("Failed to read STDIN: {e}");
        exit(EXIT_INVALID_ARGS);
    }

    let text = match String::from_utf8(buffer) {
        Ok(s) => s,
        Err(e) => {
            error!("{}: {e}", t!("main.invalidUtf8"));
            exit(EXIT_INVALID_ARGS);
        }
    };

    if text.trim().is_empty() {
        return DiscoveryInput::default();
    }

    match serde_json::from_str::<DiscoveryInput>(&text) {
        Ok(input) => input,
        Err(e) => {
            error!("{}: {e}", t!("main.invalidInput"));
            exit(EXIT_INVALID_ARGS);
        }
    }
}

/// Configures the global `tracing` subscriber based on CLI flags.
fn enable_tracing(level: &Option<TraceLevel>, format: &TraceFormat) {
    let filter = match level {
        Some(TraceLevel::Error) => "error",
        Some(TraceLevel::Warn) | None => "warn",
        Some(TraceLevel::Info) => "info",
        Some(TraceLevel::Debug) => "debug",
        Some(TraceLevel::Trace) => "trace",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    let result = match format {
        TraceFormat::Json => tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .with_writer(io::stderr)
            .try_init(),
        _ => fmt()
            .with_env_filter(env_filter)
            .with_writer(io::stderr)
            .try_init(),
    };

    if let Err(e) = result {
        eprintln!("{}: {e}", t!("tracing.unableToSetSubscriber"));
    }
}

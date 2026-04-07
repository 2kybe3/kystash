/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program comes with ABSOLUTELY NO WARRANTY!
 */

mod client;
pub mod config;
mod server;
pub mod shared;
pub mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue)]
    trace: bool,

    #[arg(long, global = true, value_name = "FILE")]
    client_config: Option<PathBuf>,

    #[arg(long, global = true, value_name = "FILE")]
    server_config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Server {
        #[command(subcommand)]
        command: server::commands::ServerCommands,
    },
    Edit,
    Upload {
        #[arg(value_name = "FILE")]
        file: PathBuf,

        #[arg(short, long)]
        server: Option<String>,
    },
    CheckServer {
        #[arg(short, long)]
        server: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(unix))]
    compile_error!(
        "This project only supports Unix-like systems (the main reasons are the missing write_at and read_at functions). Contributions for other platforms are welcome."
    );

    let cli = Cli::parse();
    utils::logging::tracing_init(cli.trace, cli.debug);

    debug!("{cli:?}");

    match cli.command {
        // Server Commands
        Commands::Server { ref command } => server::handle(command, cli.server_config).await,

        // Client Commands
        Commands::Edit => client::edit(cli.client_config).await,
        Commands::CheckServer { server } => client::check_server(cli.client_config, server).await,
        Commands::Upload { file, server } => client::upload(cli.client_config, server, file).await,
    };

    Ok(())
}

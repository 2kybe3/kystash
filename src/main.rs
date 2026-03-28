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
pub mod editor;
pub mod error;
mod logging;
mod server;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[arg(short, long, global = true, value_name = "FILE")]
    pub client_config: Option<PathBuf>,

    #[arg(short, long, global = true, value_name = "FILE")]
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
    CheckServer {
        #[arg(long)]
        server: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    logging::tracing_init(cli.debug);
    debug!("{cli:?}");

    match cli.command {
        Commands::Server { ref command } => server::handle(command, cli.server_config).await,
        Commands::CheckServer { server } => client::check_server(cli.client_config, server).await,
        Commands::Edit => client::edit(cli.client_config).await,
    };

    Ok(())
}

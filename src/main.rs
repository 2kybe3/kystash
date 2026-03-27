/*
 * kystash - A simple image/file sharing server
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
pub mod error;
mod logging;
pub mod paths;
mod server;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[arg(short, long, value_name = "FILE")]
    pub client_config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Server {
        #[command(subcommand)]
        command: server::commands::ServerCommands,

        #[arg(short, long, value_name = "FILE")]
        server_config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    logging::tracing_init(cli.debug);
    debug!("{:?}", cli);

    match cli.command {
        Commands::Server { ref command, .. } => server::handle(&cli, command).await,
    };

    Ok(())
}

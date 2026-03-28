/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ServerCommands {
    Launch,
    GenerateClientConfig {
        #[arg(short, long)]
        name: String,
    },
    GenerateServerConfig {
        #[arg(long, action = clap::ArgAction::SetTrue)]
        stdout: bool,
    },
}

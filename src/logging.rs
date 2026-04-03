/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use std::io;
use tracing_subscriber::EnvFilter;

use crate::error;

pub fn tracing_init(trace: bool, debug: bool) {
    let level = if trace {
        "trace"
    } else if debug {
        "debug"
    } else {
        "info"
    };

    let def = match format!("kystash={level}").parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("An error ocured setting up the logger. how ironic: {e}");
            error::fatal_error();
        }
    };

    let env = EnvFilter::builder()
        .with_default_directive(def)
        .from_env_lossy();

    let shared = tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_writer(io::stderr);

    if trace || debug || cfg!(debug_assertions) {
        shared
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .init();
    } else {
        shared.with_target(false).init();
    }
}

/*
 * kystash - A simple image/file sharing server
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

use tracing::level_filters::LevelFilter;

pub fn tracing_init(debug: bool) {
    if debug || cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(if debug {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            })
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .init();
    } else {
        tracing_subscriber::fmt().with_target(false).init();
    }
}

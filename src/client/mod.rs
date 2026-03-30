/*
 * kystash - A simple image/file sharing server/client
 * Copyright (C) 2026 2kybe3 <kybe@kybe.xyz>
 */

mod edit;
pub use edit::edit;
mod check_server;
pub use check_server::check_server;
mod upload;
pub use upload::upload;

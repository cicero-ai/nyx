//#![allow(warnings)]
// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::config::NyxConfig;
pub use crate::error::Error;
use falcon_cli::*;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref CONFIG: NyxConfig = config::load();
}

mod cli;
mod config;
pub mod database;
mod error;
pub mod rpc;
pub mod security;

fn main() {
    // Start daemon, if needed
    if env::args().collect::<Vec<String>>().contains(&"-d".to_string())
        && let Err(e) = rpc::launcher::start_daemon()
    {
        cli_error!("{}", e);
    }

    // Run CLI command
    let mut router = cli::boot();
    cli_run(&mut router);
}

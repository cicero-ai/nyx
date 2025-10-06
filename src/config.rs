// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::DatabaseTimeout;
use falcon_cli::*;
use std::env;
use std::process::exit;
use std::str::FromStr;

pub struct NyxConfig {
    pub dbfile: String,
    pub host: String,
    pub port: u16,
    pub timeout: Option<DatabaseTimeout>,
    pub clipboard_timeout: u64,
    pub fuse_mount_dir: String,
}

/// Gather CLI arguments, create config
pub fn load() -> NyxConfig {
    let mut config = NyxConfig::default();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    while !args.is_empty() {
        if args.len() < 2 {
            break;
        }

        // Check for value based flag
        match args[0].as_str() {
            "-f" | "--dbfile" => set_dbfile(&args[1], &mut config),
            "-h" | "--host" => config.host = args[1].to_string(),
            "-p" | "--port" => set_port(&args[1], &mut config),
            "-t" | "--timeout" => set_timeout(&args[1], &mut config),
            "-c" | "--cb-timeout" => set_clipboard_timeout(&args[1], &mut config),
            "-m" | "--mount-dir" => config.fuse_mount_dir = args[1].to_string(),
            _ => {}
        };
        args.drain(0..2);
    }

    config
}

/// Set database file
fn set_dbfile(filepath: &str, config: &mut NyxConfig) {
    config.dbfile = filepath.to_string();
}

/// Set port, after ensuring it's u16
fn set_port(port_str: &str, config: &mut NyxConfig) {
    config.port = match port_str.parse::<u16>() {
        Ok(r) => r,
        Err(_) => {
            cli_error!("ERROR: Invalid port number, {}", port_str);
            exit(1);
        }
    };
}

/// Validate and set timeout
fn set_timeout(value: &str, config: &mut NyxConfig) {
    let timeout = match DatabaseTimeout::from_str(value) {
        Ok(r) => r,
        Err(_) => {
            cli_error!("Invalid timeout value, {}", value);
            exit(1);
        }
    };
    config.timeout = Some(timeout);
}

/// Set clipboard timeout
fn set_clipboard_timeout(secs_str: &str, config: &mut NyxConfig) {
    config.clipboard_timeout = match secs_str.parse::<u64>() {
        Ok(r) => r,
        Err(_) => {
            cli_error!("ERROR: Invalid clipboard timeout value, {}", secs_str);
            exit(1);
        }
    };
}

impl Default for NyxConfig {
    fn default() -> Self {
        Self {
            dbfile: String::new(),
            host: "127.0.0.1".to_string(),
            port: 7924,
            timeout: None,
            clipboard_timeout: 120,
            fuse_mount_dir: "/tmp/nyx".to_string(),
        }
    }
}

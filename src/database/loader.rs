// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::{DatabaseTimeout, NyxDb};
use crate::security::crypto;
use crate::{CONFIG, Error};
use dirs;
use falcon_cli::*;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::{env, fs};
use zeroize::Zeroize;

pub enum LoaderResponse {
    Created((String, [u8; 32])),
    Found(String),
    NotFound,
}

/// Load and initialize the Nyx database
pub fn load() -> Result<(String, [u8; 32]), Error> {
    cli_header("Nyx");

    // Get database file
    let dbfile = match get_db_filename(true) {
        LoaderResponse::Created((dbfile, n_password)) => return Ok((dbfile, n_password)),
        LoaderResponse::Found(path) => path,
        LoaderResponse::NotFound => unreachable!(),
    };

    let n_password = match NyxDb::unlock(&dbfile) {
        Ok(r) => r,
        Err(e) => {
            cli_error!("Unable to load database, quitting.  Error: {}", e);
            exit(1);
        }
    };

    Ok((dbfile, n_password))
}

/// Get full path to database file
pub fn get_db_filename(allow_create: bool) -> LoaderResponse {
    // Check CLI args
    if !CONFIG.dbfile.is_empty() {
        return LoaderResponse::Found(CONFIG.dbfile.to_string());
    }

    // Check env variable
    if let Ok(filename) = env::var("NYX_DBFILE")
        && Path::new(&filename).exists()
    {
        return LoaderResponse::Found(filename.to_string());
    }

    // Check data directory
    let mut default_dbfile = "nyx.db".to_string();
    if let Some(mut datadir) = dirs::data_dir() {
        datadir.push("nyx/nyx.db");
        if datadir.exists() {
            return LoaderResponse::Found(datadir.to_string_lossy().into_owned());
        }
        default_dbfile = datadir.to_string_lossy().into_owned();
    }

    // User message
    cli_info!("No Nyx database found. Please specify the database location:\n");
    if allow_create {
        cli_info!(
            "Enter path to existing database or where to create new one (press Enter for default):\n"
        );
    }

    // Get file
    let dbfile = if allow_create {
        cli_get_input(
            &format!("Database Location [{}]: ", default_dbfile),
            &default_dbfile,
        )
    } else {
        cli_get_input("Database Location [nyx.db]: ", "nyx.db")
    };

    if Path::new(&dbfile).exists() {
        return LoaderResponse::Found(dbfile);
    } else if !allow_create {
        return LoaderResponse::NotFound;
    }

    // Create database
    let n_password = create_database(&dbfile);
    LoaderResponse::Created((dbfile, n_password))
}

/// Create new database
pub fn create_database(dbfile: &str) -> [u8; 32] {
    // Get password
    let mut password = cli_get_new_password(0);
    let n_password = crypto::normalize_password(&password);

    // GEt duration
    cli_info!("Lock database after inactivity (default: 1h):");
    cli_info!("    n - Never lock");
    cli_info!("    Or enter timeout: 30s, 15m, 2h, etc.\n");

    let duration: DatabaseTimeout;
    loop {
        let duration_str = cli_get_input("Timeout [1 hour]: ", "1h");
        if let Ok(dur) = DatabaseTimeout::from_str(&duration_str) {
            duration = dur;
            break;
        }
        cli_error!("Invalid duration, please try again.\n");
    }

    // Create database
    if let Err(e) = NyxDb::create(dbfile, &password, duration) {
        cli_error!("Unable to create Nyx database: {}", e);
        exit(1);
    };

    let payload = match fs::read(dbfile) {
        Ok(r) => r,
        Err(e) => {
            cli_error!("UNable to create database file: {}", e);
            exit(1);
        }
    };

    // Get mnemonic pass phrase
    let words = match crypto::get_bip39_words(&payload, &password) {
        Ok(r) => r,
        Err(e) => {
            let _ = fs::remove_file(dbfile);
            cli_error!("Unable to obtain mnemonic phrase: {}", e);
            exit(1);
        }
    };

    // Display mnemonic passphrase
    cli_header("Mnemonic Phrase");
    cli_send!(
        "Save the below 24 word phrase in a safe place, as it can be used to recover your database in the event of a lost password.\n\n"
    );
    for chunk in words.chunks(6) {
        cli_send!("    {}\n", chunk.join(" ").to_string());
    }
    cli_send!("\n");

    // Save phrase
    cli_send!(
        "Optionally, you may save the phrase to a file by specifying its location below.\n\n"
    );
    let phrase_file = cli_get_input("File Location: ", "");
    if !phrase_file.is_empty()
        && let Err(e) = fs::write(
            &phrase_file,
            format!("# Nyx Recovery Phrase\n\n{}\n", words.join(" ")),
        )
    {
        cli_error!("Unable to save phrase file: {}", e);
        exit(1);
    };

    password.zeroize();
    n_password
}

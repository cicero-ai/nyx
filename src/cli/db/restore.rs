// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::{LoaderResponse, NyxDb, loader};
use crate::security::crypto;
use bincode::config;
use falcon_cli::*;
use std::fs;

use crate::Error;

#[derive(Default)]
pub struct CliDbRestore {}

impl CliCommand for CliDbRestore {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        // Get dbfile
        cli_header("Restore Nyx Database");
        let dbfile = match loader::get_db_filename(false) {
            LoaderResponse::Found(file) => file,
            _ => {
                cli_info!("No database file found, quitting.");
                return Ok(());
            }
        };

        // GEt file contents
        let bytes = fs::read(&dbfile)?;

        cli_send!("\n");
        cli_send!("Database found at: {}\n\n", dbfile);
        cli_send!(
            "Enter the 24 word recovery phrase below that you received during database creation.\n\n"
        );
        let phrase = cli_get_input("Recovery Phrase: ", "");

        // Try to restore
        let (decrypted, master_key) = match crypto::restore_from_bip39_words(&bytes, &phrase) {
            Ok(r) => r,
            Err(_) => {
                cli_error!("Unable to restore database, invalid recovery pass phrase.");
                return Ok(());
            }
        };

        // Decode
        let (mut db, _len): (NyxDb, usize) =
            bincode::decode_from_slice(&decrypted[5..], config::standard())
                .map_err(|e| Error::Db(format!("Unable to load database: {}", e)))?;

        // Get new password
        cli_info!("Recovery phrase verified, please specify a new password below.\n\n");
        let password = cli_get_new_password(0);
        let n_password = crypto::normalize_password(&password);

        // Save database
        db.save(&dbfile, n_password, Some(master_key))?;

        cli_info!("Successfully restored Nyx database and reset password.");
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Restore Nyx Database",
            "nyx db restore [-f DBFILE]",
            "Restore a Nyx database using the 24 word recovery phrase.",
        );

        help.add_example("nyx db restore");
        help
    }
}

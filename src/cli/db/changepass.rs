// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::{LoaderResponse, NyxDb, loader};
use crate::security::crypto;
use falcon_cli::*;
use std::fs;

#[derive(Default)]
pub struct CliDbChangePass {}

impl CliCommand for CliDbChangePass {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        cli_header("Change Nyx Database Password");
        let dbfile = match loader::get_db_filename(false) {
            LoaderResponse::Found(file) => file,
            _ => {
                cli_info!("No database file found, quitting.");
                return Ok(());
            }
        };

        // Unlock
        cli_send!("Confirm your current database password:\n");
        let n_password = NyxDb::unlock(&dbfile)?;

        // Load database
        let mut db = NyxDb::load(&dbfile, n_password)?;

        // Extract master key
        let bytes = fs::read(&dbfile)?;
        let (_, master_key) = crypto::extract_master_key(&bytes, n_password)?;

        // Get new password
        cli_send!("\nSpecify the new database password:\n");
        let new_password = cli_get_new_password(0);
        let new_n_password = crypto::normalize_password(&new_password);

        // Save database
        db.save(&dbfile, new_n_password, Some(master_key))?;

        cli_send!("Successfully changed Nyx database password.\n");
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Change Database Password",
            "nyx db changepass [-f <DBFILE>]",
            "Changes the password on a Nyx database.",
        );

        help.add_example("nyx db changepass");
        help
    }
}

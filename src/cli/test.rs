// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc::launcher;
use crate::security::crypto;
use std::time::Duration;
use crate::database::{NyxDb, DatabaseTimeout};
use falcon_cli::*;
use crate::Error;

#[derive(Default)]
pub struct CliTest {}

impl CliCommand for CliTest {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        req.require_params(1)?;

        match req.args[0].as_str() {
            "createdb" => self.create_db(&req),
            _ => Err(Error::Generic(format!("Invalid action, {}", req.args[0])).into())
        }
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new("Delete Note", "nyx note rm <NAME>", "Deletes a note");

        help.add_param("NAME", "Name of the entry to delete.");
        help.add_example("nyx note rm mysite/cloudflare");
        help
    }
}

impl CliTest {
    fn create_db(&self, req: &CliRequest)  -> anyhow::Result<()> {

        // Create database
        let timeout = DatabaseTimeout::Duration(Duration::from_secs(300));
        let _db = NyxDb::create(&req.args[1], &req.args[2], timeout)?;
        let n_password = crypto::normalize_password(&req.args[2]);

        launcher::launch(&req.args[1], n_password)?;
    cli_info!("Database created at {}", req.args[1]);
        Ok(())
    }
}


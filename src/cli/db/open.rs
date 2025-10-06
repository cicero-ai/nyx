// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::loader;
use crate::rpc::{self, launcher};
use falcon_cli::*;

#[derive(Default)]
pub struct CliDbOpen {}

impl CliCommand for CliDbOpen {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        if launcher::ping() {
            cli_info!("A Nyx database is currently open, closing...");
            let _ = rpc::send::<String, bool>("db.close", &vec![]);
        }

        let (dbfile, n_password) = match loader::load() {
            Ok(r) => r,
            Err(e) => {
                cli_error!("Unable to load Nyx database: {}", e);
                return Ok(());
            }
        };

        if let Err(e) = launcher::launch(&dbfile, n_password) {
            cli_error!("Unable to launch RPC daemon: {}", e);
            return Ok(());
        }

        cli_info!("Opened Nyx database, and it's now ready for commands.");
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Open Nyx Database",
            "nyx db open [-f <DBFILE>]",
            "Opens a Nyx database",
        );

        help.add_param("DBFILE", "Optional location of database file to open.");
        help.add_example("nyx db open");
        help
    }
}

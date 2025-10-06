// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::loader;
use crate::rpc::launcher;
use falcon_cli::*;

#[derive(Default)]
pub struct CliDbCreate {}

impl CliCommand for CliDbCreate {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        // Get file location
        cli_header("Create Nyx Database");
        cli_info!("Specify the file location to create the new database.\n");
        let dbfile = cli_get_input("File Location [nyx.db]: ", "nyx.db");

        // Create database
        let n_password = loader::create_database(&dbfile);

        // Start RPC daemon
        let _ = launcher::launch(&dbfile, n_password);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create Nyx Database",
            "nyx db create",
            "Creates a new Nyx database",
        );

        help.add_example("nyx db create");
        help
    }
}

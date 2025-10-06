// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc::{self, launcher};
use falcon_cli::*;

#[derive(Default)]
pub struct CliDbClose {}

impl CliCommand for CliDbClose {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        if !launcher::ping() {
            cli_info!("No Nyx database is currently open, quitting.");
            return Ok(());
        }

        // Send RPC
        let _ = rpc::send::<String, bool>("db.close", &vec![]);

        cli_info!("Closed Nyx database.");
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Close Nyx Database",
            "nyx db close",
            "Closes any currently open Nyx database",
        );

        help.add_example("nyx db close");
        help
    }
}

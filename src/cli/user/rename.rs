// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliUserRename {}

impl CliCommand for CliUserRename {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.len() < 2 {
            cli_error!("You did not specify a source or destination to rename.");
            cli_info!("    Usage: nyx mv <SOURCE_NAME> <DEST_NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure  source exists
        cli::check_exists("user", &req.args[0], true)?;

        // Ensure destination  doesn't exist
        cli::check_exists("user", &req.args[1], false)?;

        // Rename user
        rpc::send::<String, bool>(
            "user.rename",
            &vec![req.args[0].to_string(), req.args[1].to_string()],
        )?;

        cli_info!("Renamed {} to {}\n", req.args[0], req.args[1]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help =
            CliHelpScreen::new("Rename User", "nyx mv <SOURCE> <DEST>", "Renames a user");

        help.add_param("SOURCE", "Name of existing entry to rename.");
        help.add_param("DEST", "Name of entry to rename the entry to.");
        help.add_example("nyx mv mysite/github mysite/gitlab");
        help
    }
}

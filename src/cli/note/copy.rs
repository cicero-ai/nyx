// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliNoteCopy {}

impl CliCommand for CliNoteCopy {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.len() < 2 {
            cli_error!("You did not specify a source or destination to copy.");
            cli_info!("    Usage: nyx note cp <SOURCE_NAME> <DEST_NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure  source exists
        cli::check_exists("note", &req.args[0], true)?;

        // Ensure destination  doesn't exist
        cli::check_exists("note", &req.args[1], false)?;

        // Copy item
        rpc::send::<String, bool>(
            "note.copy",
            &vec![req.args[0].to_string(), req.args[1].to_string()],
        )?;

        cli_info!("Copied {} to {}\n", req.args[0], req.args[1]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Copy Note",
            "nyx note cp <SOURCE> <DEST>",
            "Copy note to a new location",
        );

        help.add_param("SOURCE", "The source entry to copy from.");
        help.add_param("DEST", "The destination to copy the entry to.");

        help.add_example("nyx note cp mysite/pass1 anothersite/pass2");
        help
    }
}

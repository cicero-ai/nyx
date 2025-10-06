// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliNoteDelete {}

impl CliCommand for CliNoteDelete {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to delete.");
            cli_info!("    Usage: nyx note rm <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure  source exists
        cli::check_exists("note", &req.args[0], true)?;

        // Delete item
        rpc::send::<String, bool>("note.delete", &vec![req.args[0].to_string()])?;

        cli_info!("Deleted entry {}\n", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new("Delete Note", "nyx note rm <NAME>", "Deletes a note");

        help.add_param("NAME", "Name of the entry to delete.");
        help.add_example("nyx note rm mysite/cloudflare");
        help
    }
}

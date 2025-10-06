// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::Note;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliNoteShow {}

impl CliCommand for CliNoteShow {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx note show <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if item already exists
        cli::check_exists("note", &req.args[0], true)?;

        // Get item
        let note: Note = rpc::send("note.get", &vec![&req.args[0]])?;

        // Show note
        cli_header(&format!("Note: {}", req.args[0]));
        let lines: Vec<&str> = note.note.split("\n").collect();
        for line in lines {
            cli_send!("{}\n", line);
        }

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Show Note Details",
            "nyx note show <NAME>",
            "Displays the full contents of a note",
        );

        help.add_param("NAME", "Name of entry to show details of.");
        help.add_example("nyx note show mysite/cloudflare");
        help
    }
}

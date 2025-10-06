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
pub struct CliNoteNew {}

impl CliCommand for CliNoteNew {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name for the new entry.\n");
            cli_info!("    Usage: nyx note new <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure item not exists
        cli::check_exists("note", &req.args[0], false)?;

        // Get note
        let note = cli_text_editor("")?;
        if note.is_empty() {
            cli_error!("No note contents specified.");
            return Ok(());
        }
        // Instantiate item
        let note = Note {
            display_name: req.args[0].to_string(),
            note,
        };

        let note_str = serde_json::to_string(&note)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        rpc::send::<&String, bool>("note.new", &vec![&req.args[0].to_lowercase(), &note_str])?;

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create New Note",
            "nyx note new <NAME>",
            "Creates a new  note",
        );

        help.add_param(
            "NAME",
            "Name of entry to add.  Supports directory structure (eg. category/myuser)",
        );
        help.add_example("nyx new mysite/cloudflare");
        help
    }
}

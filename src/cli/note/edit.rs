// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::Note;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliNoteEdit {}

impl CliCommand for CliNoteEdit {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to edit.\n");
            cli_info!("    Usage: nyx note edit <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Get item
        let mut note: Note = rpc::send("note.get", &vec![&req.args[0]])?;

        // Edit note
        let new_note = cli_text_editor(&note.note)?;
        note.note = new_note;

        let note_str = serde_json::to_string(&note)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Edit item
        rpc::send::<&String, bool>("note.edit", &vec![&req.args[0].to_lowercase(), &note_str])?;

        cli_info!("Updated entry info for {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Edit Note",
            "nyx note edit <NAME>",
            "Edit details of a note",
        );

        help.add_param("NAME", "Name of the entry to edit.");
        help.add_example("nyx note edit mysite/cloudflare");
        help
    }
}

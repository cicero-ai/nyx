// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliNoteList {}

impl CliCommand for CliNoteList {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Get dirname
        let dirname = if !req.args.is_empty() {
            req.args[0].to_string()
        } else {
            String::new()
        };
        let start = req.get_flag("-n").unwrap_or("0".to_string());

        // Send RPC
        let entries: Vec<String> = rpc::send("note.list", &vec![&dirname.to_string(), &start])?;

        // Get table rows
        let rows = entries
            .iter()
            .enumerate()
            .map(|(x, entryname)| vec![format!("{}", x + 1), entryname.to_string()])
            .collect::<Vec<Vec<String>>>();

        // Display table
        cli_header(&format!("{}/", dirname));
        cli_display_table(&["#", "Name"], &rows);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "List Notes",
            "nyx note ls [<DIRNAME>] [-n XX]",
            "Lists all notes within directory in alphabetical order.",
        );

        help.add_param("DIRNAME", "Optional directory name to list entries from.");
        help.add_flag("-n", "Optional offset / start position of entries.");
        help.add_example("nyx note ls mysite");
        help
    }
}

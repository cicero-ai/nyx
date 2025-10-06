// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliSshKeyFind {}

impl CliCommand for CliSshKeyFind {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check
        if req.args.is_empty() {
            cli_info!("You did not specify a search string.");
            cli_info!("    Usage:  nyx ssh find <SEARCH>\n");
            return Err(CliError::MissingParams.into());
        }

        // Send RPC
        let entries: Vec<String> = rpc::send("ssh.find", &vec![req.args[0].to_string()])?;

        // Get table rows
        let rows = entries
            .iter()
            .enumerate()
            .map(|(x, entryname)| vec![format!("{}", x + 1), entryname.to_string()])
            .collect::<Vec<Vec<String>>>();

        // Display table
        cli_header(&format!("Results for {}", req.args[0]));
        cli_display_table(&["#", "Name"], &rows);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Search SSH Keys",
            "nyx ssh find <TEXT>",
            "Search all SSH keys",
        );

        help.add_param("TEXT", "The text to search all entries for.");
        help.add_example("nyx ssh find my-username");
        help
    }
}

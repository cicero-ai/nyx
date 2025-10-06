// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliUserList {}

impl CliCommand for CliUserList {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Get dirname
        let dirname = if !req.args.is_empty() {
            req.args[0].to_string()
        } else {
            String::new()
        };
        let start = req.get_flag("-n").unwrap_or("0".to_string());

        // Send RPC
        let users: Vec<String> = rpc::send("user.list", &vec![&dirname.to_string(), &start])?;

        // Get table rows
        let rows = users
            .iter()
            .enumerate()
            .map(|(x, username)| vec![format!("{}", x + 1), username.to_string()])
            .collect::<Vec<Vec<String>>>();

        // Display table
        cli_header(&format!("{}/", dirname));
        cli_display_table(&["#", "Name"], &rows);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "List Users",
            "nyx ls [<DIRNAME>] [-n XX]",
            "Lists all users within directory in alphabetical order.",
        );

        help.add_param("DIRNAME", "Optional directory name to list entries from.");
        help.add_flag("-n", "Optional offset / start position of entries.");
        help.add_example("nyx ls mysite");
        help
    }
}

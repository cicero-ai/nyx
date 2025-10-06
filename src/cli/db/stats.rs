// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::DbStats;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliDbStats {}

impl CliCommand for CliDbStats {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        // Get  stats
        let stats: DbStats = rpc::send::<String, DbStats>("db.stats", &vec![])?;

        // Set data
        let data = indexmap! {
            "Db File: " => stats.dbfile.to_string(),
            "Users: " => format!("{} entries, {} dirs", stats.users.0, stats.users.1),
            "OTP: " => format!("{} entries, {} dirs", stats.oauth.0, stats.oauth.1),
            "SSH Keys: " => format!("{} entries, {} dirs", stats.ssh_keys.0, stats.ssh_keys.1),
            "Strings: " => format!("{} entries, {} dirs", stats.strings.0, stats.strings.1),
            "Notes: " => format!("{} entries, {} dirs", stats.notes.0, stats.notes.1)
        };

        // Display
        cli_header("Nyx Database Stats");
        cli_display_array(&data);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Database Stats",
            "nyx db stats",
            "Displays overall database statistics",
        );

        help.add_example("nyx db stats");
        help
    }
}

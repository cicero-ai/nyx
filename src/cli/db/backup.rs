// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::DbStats;
use crate::rpc;
use falcon_cli::*;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct CliDbBackup {}

impl CliCommand for CliDbBackup {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {
        // Get stats
        let stats: DbStats = rpc::send::<String, DbStats>("db.stats", &vec![])?;

        // Get file location
        cli_header("Backup Nyx Database");
        cli_info!("Specify the file location to backup the database to.\n");
        let dbfile = cli_get_input("File Location [nyx.db]: ", "nyx.db");

        // Check parent dir
        if let Some(parent) = Path::new(&dbfile).parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        // Copy
        fs::copy(&stats.dbfile, &dbfile)?;
        cli_info!("Successfully backed up Nyx database to: {}", dbfile);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Backup Nyx Database",
            "nyx db backup",
            "Creates a backup copy of an existing Nyx database",
        );

        help.add_example("nyx db backup");
        help
    }
}

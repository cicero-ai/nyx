// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use crate::database::HistoryItem;
use crate::rpc;
use chrono::{DateTime, Utc};
use falcon_cli::*;

#[derive(Default)]
pub struct CliDbHistory {}

impl CliCommand for CliDbHistory {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        let n = if let Some(n_str) = req.get_flag("n") {
            n_str
                .parse::<usize>()
                .map_err(|e| Error::Validate(format!("Invalid start number: {}", e)))?
        } else {
            0
        };

        // Send RPC
        let entries: Vec<HistoryItem> = rpc::send("db.history", &vec![n])?;

        // Get table rows
        let rows = entries
            .iter()
            .enumerate()
            .map(|(x, item)| {
                let datetime = DateTime::<Utc>::from_timestamp(item.timestamp as i64, 0).unwrap();
                let time_str = datetime.format("%b %d, %Y %H:%M:%S").to_string();

                let num = format!("{}", (x + 1 + n));
                vec![
                    num,
                    time_str,
                    item.action.to_string(),
                    item.data_type.to_string(),
                    item.source.to_string(),
                    item.dest.to_string(),
                ]
            })
            .collect::<Vec<Vec<String>>>();

        // Display table
        cli_header("History");
        cli_display_table(&["#", "Date", "Action", "Type", "Source", "Target"], &rows);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "List Notes",
            "nyx note ls [<DIRNAME>]",
            "Lists all notes within directory in alphabetical order.",
        );

        help.add_param("DIRNAME", "Optional directory name to list entries from.");
        help.add_example("nyx note ls mysite");
        help
    }
}

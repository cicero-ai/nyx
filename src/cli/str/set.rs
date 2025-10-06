// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::StrItem;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliStrSet {}

impl CliCommand for CliStrSet {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.len() < 2 {
            cli_error!("You did not specify a name for the new entry.\n");
            cli_info!("    Usage: nyx set <NAME> <VALUE>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure item not exists
        cli::check_exists("str", &req.args[0], false)?;

        // Instantiate item
        let item = StrItem {
            display_name: req.args[0].to_string(),
            value: req.args[1].to_string(),
        };

        let item_str = serde_json::to_string(&item)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        rpc::send::<&String, bool>("str.set", &vec![&req.args[0].to_lowercase(), &item_str])?;
        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create New String",
            "nyx set <NAME> <VALUE>",
            "Creates a new string entry.",
        );

        help.add_param(
            "NAME",
            "Name of entry to add.  Supports directory structure (eg. category/myuser)",
        );
        help.add_param("VALUE", "Value of the string to add.");
        help.add_example("nyx set some-apikey abc12345apikey");
        help
    }
}

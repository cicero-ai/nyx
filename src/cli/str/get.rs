// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::cli::clipboard;
use crate::database::StrItem;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliStrGet {}

impl CliCommand for CliStrGet {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx get <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if entry already exists
        cli::check_exists("str", &req.args[0], true)?;

        // Get entry
        let item: StrItem = rpc::send("str.get", &vec![&req.args[0], &"1".to_string()])?;

        // Copy to clipboard
        clipboard::copy(&item.value)?;

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Copy String",
            "nyx get <NAME>",
            "Copies value of string to clipboard",
        );

        help.add_param("NAME", "Name of entry to copy.");
        help.add_example("nyx get mysite/cloudflare");
        help
    }
}

// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::User;
use crate::rpc;
use crate::security::password;
use falcon_cli::*;

#[derive(Default)]
pub struct CliUserNew {}

impl CliCommand for CliUserNew {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name for the new user.\n");
            cli_info!("    Usage: nyx new <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure user not exists
        cli::check_exists("user", &req.args[0], false)?;

        // Get user info
        cli_header("Create New User");
        cli_info!("Enter the new user information below.  Leave blank to omit a field.\n");
        let username = cli_get_input("Username: ", "");
        let password = password::from_cli(false);
        let url = cli_get_input("URL: ", "");
        let notes = cli_get_multiline_input("Additional Notes");

        // Instantiate user
        let user = User {
            display_name: req.args[0].to_string(),
            username,
            password,
            url,
            notes,
        };

        let user_str = serde_json::to_string(&user)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create user
        rpc::send::<&String, bool>("user.new", &vec![&req.args[0].to_lowercase(), &user_str])?;

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create New User",
            "nyx new <NAME>",
            "Creates a new user.  For password, you may use 'g' to generate a 24 character password or 'gXX' to generate a password of a desired length.",
        );

        help.add_param(
            "NAME",
            "Name of entry to add.  Supports directory structure (eg. category/myuser)",
        );
        help.add_example("nyx new mysite/cloudflare");
        help
    }
}

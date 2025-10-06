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
pub struct CliUserEdit {}

impl CliCommand for CliUserEdit {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to edit.\n");
            cli_info!("    Usage: nyx edit <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure user not exists
        cli::check_exists("user", &req.args[0], true)?;

        // Get user
        let mut user: User = rpc::send("user.get", &vec![&req.args[0]])?;

        // Get user info
        cli_header(&format!("Edit {}", req.args[0]));
        cli_info!("Enter the new user information below.  Leave blank to skip a field.\n");

        let username = cli_get_input("Username: ", "");
        if !username.is_empty() {
            user.username = username;
        }

        let password = password::from_cli(true);
        if !password.is_empty() {
            user.password = password;
        }

        let url = cli_get_input("URL: ", "");
        if !url.is_empty() {
            user.url = url;
        }

        let notes = cli_get_multiline_input("Additional Notes");
        if !notes.is_empty() {
            user.notes = notes;
        }

        let user_str = serde_json::to_string(&user)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Edit user
        rpc::send::<&String, bool>("user.edit", &vec![&req.args[0], &user_str])?;

        cli_info!("Updated user info for {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new("Edit User", "nyx edit <NAME>", "Edit a user's details.");

        help.add_param("NAME", "Name of the entry to edit.");
        help.add_example("nyx edit mysite/cloudflare");
        help
    }
}

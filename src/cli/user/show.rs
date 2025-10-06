// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::User;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliUserShow {}

impl CliCommand for CliUserShow {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of a user");
            cli_info!("    Usage: nyx show <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if user already exists
        cli::check_exists("user", &req.args[0], true)?;

        // Get user
        let user: User = rpc::send("user.get", &vec![&req.args[0]])?;

        // Get vector
        let userdata = indexmap! {
            "Username:" => user.username.to_string(),
            "Password:" => user.password.to_string(),
            "URL:" => user.url.to_string(),
            "Notes:" => user.notes.to_string()
        };

        // Show user info
        cli_header(&format!("User: {}", req.args[0]));
        cli_display_array(&userdata);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Show User Details",
            "nyx show <NAME>",
            "Displays all details on user",
        );

        help.add_param("NAME", "Name of entry to show details of.");
        help.add_example("nyx show mysite/cloudflare");
        help
    }
}

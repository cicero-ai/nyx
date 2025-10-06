// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli::{self, clipboard};
use crate::database::User;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliUserXp {}

impl CliCommand for CliUserXp {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of a user");
            cli_info!("    Usage: nyx xp <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if user already exists
        cli::check_exists("user", &req.args[0], true)?;

        // Get user
        let user: User = rpc::send("user.get", &vec![&req.args[0], &"1".to_string()])?;

        // Copy to clipboard
        clipboard::copy(&user.password)?;
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Copy User Password",
            "nyx xp <NAME>",
            "Copy user password to clipboard",
        );

        help.add_param("NAME", "Name of entry to copy from.");
        help.add_example("nyx xp mysite/cloudflare");
        help
    }
}

// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::Oauth;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliOtpShow {}

impl CliCommand for CliOtpShow {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx otp show <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if item already exists
        cli::check_exists("otp", &req.args[0], true)?;

        // Get item
        let otp: Oauth = rpc::send("otp.get", &vec![&req.args[0]])?;

        // Get vector
        let data = indexmap! {
            "Secret Code:" => otp.secret_code.to_string(),
            "URL:" => otp.url.to_string(),
            "Recovery Keys:" => otp.recovery_keys.to_string()
        };

        // Show item info
        cli_header(&format!("OTP: {}", req.args[0]));
        cli_display_array(&data);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Show OTP Entry Details",
            "nyx otp show <NAME>",
            "Displays all details on OTP entry",
        );

        help.add_param("NAME", "Name of entry to show details of.");
        help.add_example("nyx otp show mysite/cloudflare");
        help
    }
}

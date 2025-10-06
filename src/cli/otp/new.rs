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
pub struct CliOtpNew {}

impl CliCommand for CliOtpNew {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name for the new entry.\n");
            cli_info!("    Usage: nyx otp new <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure item not exists
        cli::check_exists("otp", &req.args[0], false)?;

        // Get item info
        cli_header("Create New OTP Entry");
        cli_info!("Enter the new OTP information below.  Leave blank to omit a field.\n");
        let secret_code = cli_get_input("Secret Code: ", "");
        let url = cli_get_input("URL: ", "");
        let recovery_keys = cli_get_multiline_input("Recovery Keys");

        // Instantiate item
        let otp = Oauth {
            display_name: req.args[0].to_string(),
            secret_code,
            url,
            recovery_keys,
        };

        let otp_str = serde_json::to_string(&otp)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        rpc::send::<&String, bool>("otp.new", &vec![&req.args[0].to_lowercase(), &otp_str])?;

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Create New OTP Entry",
            "nyx otp new <NAME>",
            "Creates a new OTP entry.",
        );

        help.add_param(
            "NAME",
            "Name of entry to add.  Supports directory structure (eg. category/myuser)",
        );
        help.add_example("nyx new mysite/cloudflare");
        help
    }
}

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
pub struct CliOtpEdit {}

impl CliCommand for CliOtpEdit {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to edit.\n");
            cli_info!("    Usage: nyx otp edit <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure item exists
        cli::check_exists("otp", &req.args[0], true)?;

        // Get item
        let mut otp: Oauth = rpc::send("otp.get", &vec![&req.args[0]])?;

        // Get item info
        cli_header(&format!("Edit {}", req.args[0]));
        cli_info!("Enter the new OTP information below.  Leave blank to skip a field.\n");

        let secret_code = cli_get_input("Secret Code: ", "");
        if !secret_code.is_empty() {
            otp.secret_code = secret_code;
        }

        let url = cli_get_input("URL: ", "");
        if !url.is_empty() {
            otp.url = url;
        }

        let recovery_keys = cli_get_multiline_input("Recovery Keys");
        if !recovery_keys.is_empty() {
            otp.recovery_keys = recovery_keys;
        }

        let otp_str = serde_json::to_string(&otp)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Edit item
        rpc::send::<&String, bool>("otp.edit", &vec![&req.args[0].to_lowercase(), &otp_str])?;

        cli_info!("Updated entry info for {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Edit OTP Entry",
            "nyx otp edit <NAME>",
            "Edit details of an OTP entry.",
        );

        help.add_param("NAME", "Name of the entry to edit.");
        help.add_example("nyx otp edit mysite/cloudflare");
        help
    }
}

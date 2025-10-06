// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli::{self, clipboard};
use crate::database::Oauth;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliOtpXr {}

impl CliCommand for CliOtpXr {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify an entry name.");
            cli_info!("    Usage: nyx otp xr <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if entry already exists
        cli::check_exists("otp", &req.args[0], true)?;

        // Get entry
        let otp: Oauth = rpc::send("otp.get", &vec![&req.args[0], &"1".to_string()])?;

        // Copy to clipboard
        clipboard::copy(&otp.recovery_keys)?;

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Copy OTP Entry Recovery COdes",
            "nyx otp xr <NAME>",
            "Copy OTP entry recovery codes to clipboard.",
        );

        help.add_param("NAME", "Name of entry to copy from.");
        help.add_example("nyx otp xr mysite/cloudflare");
        help
    }
}

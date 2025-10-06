// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::cli::clipboard;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliOtpGenerate {}

impl CliCommand for CliOtpGenerate {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx otp show <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if entry already exists
        cli::check_exists("otp", &req.args[0], true)?;

        // Generate otp
        let otp: String = rpc::send("otp.generate", &vec![&req.args[0]])?;

        // copy to clipboard
        clipboard::copy(&otp)?;
        cli_info!("{}", otp);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Generate OTP",
            "nyx otp <NAME>",
            "Generates 6 digit OTP code for authentication.",
        );

        help.add_param("NAME", "Name of OTP entry to generate code for.");
        help.add_example("nyx otp mysite/cloudflare");
        help
    }
}
